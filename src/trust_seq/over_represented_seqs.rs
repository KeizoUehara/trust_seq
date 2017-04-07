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
}
impl QCModule for OverRepresentedSeqs {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        writeln!(writer, "#Sequence\tCount\tPercentage\tPossible Source");
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
                     possible_source);
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
