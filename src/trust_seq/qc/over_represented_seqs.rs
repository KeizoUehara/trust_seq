use crate::trust_seq::contaminant::find_contaminant;
use crate::trust_seq::contaminant::Contaminant;
use crate::trust_seq::contaminant_list::CONTAMINANT_LIST;
use crate::trust_seq::qc::{QCModule, QCReport, QCResult};
use crate::trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use crate::trust_seq::utils::Sequence;
use serde_json::map::Map;
use serde_json::value;
use serde_json::Value;
use std::collections::hash_map::HashMap;
use std::io::BufReader;
use std::io::Write;

const OBSERVATION_CUTOFF: usize = 100000;

pub struct OverRepresentedSeqs<'a> {
    config: &'a TrustSeqConfig,
    count: u64,
    unique_sequence_count: usize,
    count_at_unique_limit: u64,
    frozen: bool,
    sequences: HashMap<String, u32>,
}
#[derive(Serialize)]
struct OverRepresentedReport {
    status: QCResult,
    over_represented: Vec<OverRepresentedSeq>,
}
#[derive(Serialize)]
struct OverRepresentedSeq {
    seq: String,
    count: u32,
    percentage: f64,
    possible_source: String,
}
const DUP_LEVEL_LABELS: [(usize, &'static str); 16] = [
    (0, "1"),
    (1, "2"),
    (2, "3"),
    (3, "4"),
    (4, "5"),
    (5, "6"),
    (6, "7"),
    (7, "8"),
    (8, "9"),
    (9, ">10"),
    (49, ">50"),
    (99, ">100"),
    (499, "500"),
    (999, ">1k"),
    (4999, ">5k"),
    (9999, ">10k"),
];
impl<'a> OverRepresentedSeqs<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> OverRepresentedSeqs<'a> {
        return OverRepresentedSeqs {
            config: config,
            count: 0,
            unique_sequence_count: 0,
            count_at_unique_limit: 0,
            frozen: false,
            sequences: HashMap::new(),
        };
    }
}
#[derive(Serialize)]
struct DuplicationLevelReport {
    status: QCResult,
    total_dedup_percentage: f64,
    duplication_levels: Vec<DuplicationLevel>,
}
#[derive(Serialize)]
struct DuplicationLevel {
    label: &'static str,
    deduplicated_percentage: f64,
    total_percentage: f64,
}

fn get_corrected_count(
    count_at_limit: u64,
    total_count: u64,
    duplication_level: u64,
    number_of_observations: u64,
) -> f64 {
    if count_at_limit == total_count {
        return number_of_observations as f64;
    }
    if total_count - number_of_observations < count_at_limit {
        return number_of_observations as f64;
    }

    let mut p_not_seeing_at_limit = 1f64;
    for i in 0..count_at_limit {
        p_not_seeing_at_limit =
            ((total_count - i) - duplication_level) as f64 / (total_count - i) as f64;
    }
    return number_of_observations as f64 / (1.0 - p_not_seeing_at_limit);
}
fn calculate_report(
    over_represented_seqs: &OverRepresentedSeqs,
) -> Result<DuplicationLevelReport, TrustSeqErr> {
    let mut deduplicated_percentages: [f64; 16] = [0.0; 16];
    let mut total_percentages: [f64; 16] = [0.0; 16];
    let mut collated_counts: HashMap<u32, u32> = HashMap::new();
    for count in over_represented_seqs.sequences.values() {
        let c = collated_counts.entry(*count).or_insert(0);
        *c += 1;
    }
    let mut corrected_counts: HashMap<u32, f64> = HashMap::new();
    for (dup_level, count) in &collated_counts {
        corrected_counts.insert(
            *dup_level,
            get_corrected_count(
                over_represented_seqs.count_at_unique_limit,
                over_represented_seqs.count,
                *dup_level as u64,
                *count as u64,
            ),
        );
    }
    let mut dedup_total: f64 = 0.0;
    let mut row_total: f64 = 0.0;
    for (dl, c) in &corrected_counts {
        let dup_level = *dl as f64;
        let count = *c as f64;
        dedup_total += count;
        row_total += count * dup_level;
        let mut dup_slot: usize = (*dl - 1) as usize;
        for i in 0..DUP_LEVEL_LABELS.len() {
            let idx = DUP_LEVEL_LABELS.len() - i - 1;
            if dup_level > DUP_LEVEL_LABELS[idx].0 as f64 {
                dup_slot = idx;
                break;
            }
        }
        deduplicated_percentages[dup_slot] += count;
        total_percentages[dup_slot] += count * dup_level;
    }
    let mut vecs: Vec<DuplicationLevel> = Vec::new();
    for idx in 0..deduplicated_percentages.len() {
        vecs.push(DuplicationLevel {
            label: DUP_LEVEL_LABELS[idx].1,
            deduplicated_percentage: deduplicated_percentages[idx] * 100.0 / dedup_total,
            total_percentage: total_percentages[idx] * 100.0 / row_total,
        });
    }
    return Ok(DuplicationLevelReport {
        status: QCResult::Pass,
        total_dedup_percentage: dedup_total / row_total * 100.0,
        duplication_levels: vecs,
    });
}
impl QCReport for DuplicationLevelReport {
    fn get_name(&self) -> &'static str {
        return "Sequence Duplication Levels";
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        writeln!(writer, "#Total Deduplicated Percentage")?;
        writeln!(writer, "{}", self.total_dedup_percentage)?;
        writeln!(
            writer,
            "#Duplication Level\tPercentage of deduplicated\tPercentage of total\n"
        )?;
        for seq in &self.duplication_levels {
            writeln!(
                writer,
                "{}\t{}\t{}",
                seq.label, seq.deduplicated_percentage, seq.total_percentage
            )?;
        }
        return Ok(());
    }
}
impl QCReport for OverRepresentedReport {
    fn get_name(&self) -> &'static str {
        return "Sequence Duplication Levels";
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(&self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        writeln!(writer, "#Sequence\tCount\tPercentage\tPossible Source")?;
        for seq in &self.over_represented {
            writeln!(
                writer,
                "{}\t{}\t{}\t{}",
                seq.seq, seq.count, seq.percentage, seq.possible_source
            )?;
        }
        return Ok(());
    }
}
impl<'a> QCModule for OverRepresentedSeqs<'a> {
    fn calculate(&self, reports: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        reports.push(Box::new(calculate_report(self)?));
        let mut seqs: Vec<OverRepresentedSeq> = Vec::new();
        let cons = Contaminant::load_contaminants(BufReader::new(CONTAMINANT_LIST.as_bytes()));
        for (sequence, count) in &self.sequences {
            let percantage: f64 = *count as f64 * 100.0 / self.count as f64;

            if 0.1 < percantage {
                let possible_source = match find_contaminant(&cons, sequence.as_bytes()) {
                    Some(hit) => format!(
                        "{} ({}% over {} bp) {}",
                        hit.contaminant.name, hit.percent_id, hit.length, hit.direction
                    ),
                    None => "No Hit".to_string(),
                };
                seqs.push(OverRepresentedSeq {
                    seq: sequence.to_string(),
                    count: *count,
                    percentage: percantage,
                    possible_source: possible_source,
                });
            }
        }

        seqs.sort_by(|a, b| b.count.cmp(&a.count));

        let max_percant = seqs.get(0).map_or(0.0, |s| s.percentage);
        let error_th = self.config.module_config.get("overrepresented:error");
        let warn_th = self.config.module_config.get("overrepresented:warn");
        let status = if max_percant > error_th {
            QCResult::Fail
        } else if max_percant > warn_th {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        reports.push(Box::new(OverRepresentedReport {
            status: status,
            over_represented: seqs,
        }));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        self.count += 1;
        if !self.frozen {
            self.count_at_unique_limit = self.count;
        }
        let seq2: String = if seq.sequence.len() > 75 {
            String::from_utf8_lossy(&(seq.sequence[..50])).to_string()
        } else {
            String::from_utf8_lossy(seq.sequence).to_string()
        };
        if self.sequences.contains_key(&seq2) {
            *self.sequences.get_mut(&seq2).unwrap() += 1;
        } else {
            if !self.frozen {
                self.sequences.insert(seq2, 1);
                if self.sequences.len() == OBSERVATION_CUTOFF {
                    self.frozen = true;
                }
            }
        }
    }
}
