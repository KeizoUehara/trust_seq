use std::io::Write;
use std::f64;
use serde_json::value;
use serde_json::value::Value;
use trust_seq::group::BaseGroup;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;
use trust_seq::qc::{QCModule, QCResult};

pub struct PerBaseSequenceContent<'a> {
    config: &'a TrustSeqConfig,
    counts: [Vec<u64>; 4],
    report: Option<PerBaseSequenceReport>,
}
#[derive(Serialize)]
struct PerBaseSequenceReport {
    status: QCResult,
    group: Vec<BaseGroup>,
    percents: Vec<[f64; 4]>,
}
impl<'a> PerBaseSequenceContent<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerBaseSequenceContent<'a> {
        return PerBaseSequenceContent {
                   config: config,
                   counts: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
                   report: None,
               };
    }
}

impl<'a> QCModule for PerBaseSequenceContent<'a> {
    fn get_name(&self) -> &'static str {
        return "Per base sequence content";
    }
    fn get_status(&self) -> QCResult {
        return self.report.as_ref().unwrap().status;
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        let vals = self.report.as_ref().unwrap();
        write!(writer, "#Base\tG\tA\tT\tC\n")?;
        for (idx, group) in vals.group.iter().enumerate() {
            if group.lower_count == group.upper_count {
                write!(writer,
                       "{}\t{}\t{}\t{}\t{}\n",
                       group.lower_count,
                       vals.percents[idx][0],
                       vals.percents[idx][1],
                       vals.percents[idx][2],
                       vals.percents[idx][3])?;
            } else {
                write!(writer,
                       "{}-{}\t{}\t{}\t{}\t{}\n",
                       group.lower_count,
                       group.upper_count,
                       vals.percents[idx][0],
                       vals.percents[idx][1],
                       vals.percents[idx][2],
                       vals.percents[idx][3])?;
            }
        }
        return Ok(());
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        let report = self.report.as_ref().unwrap();
        return Ok(value::to_value(&report)?);
    }
    fn calculate(&mut self) -> Result<(), TrustSeqErr> {
        let seq_len = self.counts[0].len();
        let groups = BaseGroup::make_base_groups(&self.config.group_type, seq_len);
        let mut percents: Vec<[f64; 4]> = Vec::new();
        let mut max_gc_diff: f64 = 0.0;
        let mut max_at_diff: f64 = 0.0;
        for group in &groups {
            let mut total: u64 = 0;
            let mut counts: [u64; 4] = [0; 4];
            for pos_idx in (group.lower_count - 1)..group.upper_count {
                for base_idx in 0..4 {
                    counts[base_idx] += self.counts[base_idx][pos_idx];
                    total += self.counts[base_idx][pos_idx];
                }
            }
            println!("counts={:?}", counts);;
            let percent = [counts[0] as f64 * 100.0 / total as f64,
                           counts[1] as f64 * 100.0 / total as f64,
                           counts[2] as f64 * 100.0 / total as f64,
                           counts[3] as f64 * 100.0 / total as f64];
            println!("percent={:?}", percent);;
            max_gc_diff = max_gc_diff.max((percent[3] - percent[0]).abs());
            max_at_diff = max_at_diff.max((percent[2] - percent[1]).abs());
            percents.push(percent);
        }
        let error_th = self.config.module_config.get("sequence:error");
        let warn_th = self.config.module_config.get("sequence:warn");
        let status = if max_gc_diff > error_th || max_at_diff > error_th {
            QCResult::fail
        } else if max_gc_diff > warn_th || max_at_diff > warn_th {
            QCResult::warn
        } else {
            QCResult::pass
        };
        self.report = Some(PerBaseSequenceReport {
                               status: status,
                               group: groups,
                               percents: percents,
                           });
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        if self.counts[0].len() < seq.sequence.len() {
            for i in self.counts[0].len()..seq.sequence.len() {
                for count in &mut self.counts {
                    count.push(0);
                }
            }
        }
        for (idx, ch) in seq.sequence.iter().enumerate() {
            let b = match *ch as char {
                'G' => 0,
                'g' => 0,
                'A' => 1,
                'a' => 1,
                'T' => 2,
                't' => 2,
                'C' => 3,
                'c' => 3,
                _ => 4,
            };
            if b < 4 {
                self.counts[b][idx] += 1;
            }
        }
    }
}
