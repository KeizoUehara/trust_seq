use std::io::Write;
use std::io::Result;
use std::borrow::Borrow;
use std::collections::hash_map::HashMap;
use trust_seq::gc_model::GCModel;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use std::io::BufReader;
use trust_seq::contaminant::Contaminant;
use trust_seq::contaminant::ContaminantHit;
use trust_seq::contaminant_list::CONTAMINANT_LIST;

const OBSERVATION_CUTOFF: usize = 100000;

pub struct OverRepresentedSeqs {
    count: u64,
    unique_sequence_count: usize,
    count_at_unique_limit: u64,
    frozen: bool,
    sequences: HashMap<String, u32>,
}
struct OverRepresentedSeq {
    seq: String,
    count: u32,
    percentage: f64,
}
const DUP_LEVEL_LABELS: [(usize, &'static str); 16] = [(0, "1"),
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
                                                       (9999, ">10k")];
impl OverRepresentedSeqs {
    pub fn new() -> OverRepresentedSeqs {
        return OverRepresentedSeqs {
                   count: 0,
                   unique_sequence_count: 0,
                   count_at_unique_limit: 0,
                   frozen: false,
                   sequences: HashMap::new(),
               };
    }
    fn print_duplication_level(&self, writer: &mut Write) -> Result<()> {
        let mut deduplicated_percentages: [f64; 16] = [0.0; 16];
        let mut total_percentages: [f64; 16] = [0.0; 16];
        let mut collated_counts: HashMap<u32, u32> = HashMap::new();
        for count in self.sequences.values() {
            let c = collated_counts.entry(*count).or_insert(0);
            *c += 1;
        }
        let mut corrected_counts: HashMap<u32, f64> = HashMap::new();
        for (dup_level, count) in &collated_counts {
            corrected_counts.insert(*dup_level,
                                    get_corrected_count(self.count_at_unique_limit,
                                                        self.count,
                                                        *dup_level as u64,
                                                        *count as u64));
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
        writeln!(writer,
                 "#Total Deduplicated Percentage\t{}",
                 dedup_total / row_total * 100.0)?;
        writeln!(writer,
                 "#Duplication Level\tPercentage of deduplicated\tPercentage of total")?;
        for idx in 0..deduplicated_percentages.len() {
            writeln!(writer,
                     "{}\t{}\t{}",
                     DUP_LEVEL_LABELS[idx].1,
                     deduplicated_percentages[idx] * 100.0 / dedup_total,
                     total_percentages[idx] * 100.0 / row_total)?;
        }
        return Ok(());
    }
}
fn get_corrected_count(count_at_limit: u64,
                       total_count: u64,
                       duplication_level: u64,
                       number_of_observations: u64)
                       -> f64 {
    if count_at_limit == total_count {
        return number_of_observations as f64;
    }
    if total_count - number_of_observations < count_at_limit {
        return number_of_observations as f64;
    }

    let mut p_not_seeing_at_limit = 1f64;
    for i in 0..count_at_limit {
        p_not_seeing_at_limit = ((total_count - i) - duplication_level) as f64 /
                                (total_count - i) as f64;
    }
    return number_of_observations as f64 / (1.0 - p_not_seeing_at_limit);
}
impl QCModule for OverRepresentedSeqs {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        self.print_duplication_level(writer)?;
        writeln!(writer, "#Sequence\tCount\tPercentage\tPossible Source")?;
        let mut seqs: Vec<OverRepresentedSeq> = Vec::new();
        for (sequence, count) in &self.sequences {
            let percantage: f64 = *count as f64 * 100.0 / self.count as f64;
            if 0.1 < percantage {
                seqs.push(OverRepresentedSeq {
                              seq: sequence.to_string(),
                              count: *count,
                              percentage: percantage,
                          });
            }
        }
        seqs.sort_by(|a, b| b.count.cmp(&a.count));
        let cons = Contaminant::load_contaminants(BufReader::new(CONTAMINANT_LIST.as_bytes()));
        for s in seqs {
            let mut hit_len = 0;
            let mut possible_source = "No Hit".to_string();
            for con in &cons {
                if let Some(hit) = con.find_match(s.seq.as_bytes()) {
                    if hit_len < hit.length {
                        hit_len = hit.length;
                        possible_source = format!("{} ({}% over {} bp) {}",
                                                  hit.contaminant.name,
                                                  hit.percent_id,
                                                  hit.length,
                                                  hit.direction);

                    }
                }

            }
            writeln!(writer,
                     "{}\t{}\t{}\t{}",
                     s.seq,
                     s.count,
                     s.percentage,
                     possible_source)?;
        }
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
