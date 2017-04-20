use std::io::Write;
use std::cmp;
use serde_json::value;
use serde_json::value::Value;
use trust_seq::group::BaseGroup;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;
use trust_seq::qc::{QCModule, QCResult};

pub struct SequenceLengthDistribution<'a> {
    config: &'a TrustSeqConfig,
    length_counts: Vec<u64>,
    report: Option<SequenceLengthReport>,
}
#[derive(Serialize)]
struct SequenceLengthReport {
    status: QCResult,
    length_counts: Vec<GroupValue<u64>>,
}
impl<'a> SequenceLengthDistribution<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> SequenceLengthDistribution<'a> {
        return SequenceLengthDistribution {
                   config: config,
                   length_counts: Vec::new(),
                   report: None,
               };
    }
}
fn get_min_max_idx(vec: &Vec<u64>) -> (usize, usize) {
    let mut min: i32 = -1;
    let mut max: usize = 0;
    for (idx, val) in vec.iter().enumerate() {
        if *val > 0 {
            if min < 0 {
                min = idx as i32;
            }
            max = idx;
        }
    }
    return (min as usize, max);
}
#[derive(Serialize)]
struct GroupValue<V> {
    lower_count: usize,
    upper_count: usize,
    value: V,
}
fn calc_interval(width: usize) -> usize {
    let mut base: usize = 1;
    let divisions = [1, 2, 5];
    loop {
        for division in divisions.iter() {
            let interval = base * division;
            if width / interval <= 50 {
                return interval;
            }
        }
        base *= 10;
    }
}
fn get_size_distribution(min: usize, max: usize) -> (usize, usize) {
    let interval = calc_interval(max - min);
    let base_div = min / interval;
    return (base_div * interval, interval);
}
impl<'a> QCModule for SequenceLengthDistribution<'a> {
    fn get_name(&self) -> &'static str {
        return "Sequence Length Distribution";
    }
    fn get_status(&self) -> QCResult {
        return self.report.as_ref().unwrap().status;
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        let report = self.report.as_ref().unwrap();
        return Ok(value::to_value(&report)?);
    }

    fn calculate(&mut self) -> Result<(), TrustSeqErr> {
        let (mut min_len, mut max_len) = get_min_max_idx(&self.length_counts);
        let is_same_length = max_len == min_len;
        //  We put one extra category either side of the actual size
        min_len -= 1;
        max_len += 1;
        let (start, interval) = get_size_distribution(min_len, max_len);
        let mut current_pos = start;
        let mut counts: Vec<GroupValue<u64>> = Vec::new();
        while current_pos <= max_len {
            let max_pos = cmp::min(max_len, current_pos + interval - 1);
            let mut count = 0;
            for idx in current_pos..(max_pos + 1) {
                if idx < self.length_counts.len() {
                    count += self.length_counts[idx];
                }
            }
            counts.push(GroupValue {
                            lower_count: current_pos,
                            upper_count: max_pos,
                            value: count,
                        });
            current_pos += interval;
        }
        let error_th = self.config.module_config.get("sequence_length:error");
        let warn_th = self.config.module_config.get("sequence_length:warn");
        let status = if error_th != 0.0 && self.length_counts[0] > 0 {
            QCResult::fail
        } else if warn_th != 0.0 && is_same_length {
            QCResult::warn
        } else {
            QCResult::pass
        };
        self.report = Some(SequenceLengthReport {
                               status: status,
                               length_counts: counts,
                           });
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        writeln!(writer, "#Length Count")?;
        let report = self.report.as_ref().unwrap();

        for value in &report.length_counts {
            if value.lower_count == value.upper_count {
                writeln!(writer, "{}\t{}", value.lower_count, value.value)?;
            } else {
                writeln!(writer,
                         "{}-{}\t{}",
                         value.lower_count,
                         value.upper_count,
                         value.value)?;
            }
        }
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let len = seq.sequence.len();
        if self.length_counts.len() < len + 2 {
            self.length_counts.resize(len + 2, 0);
        }
        self.length_counts[len] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_interval() {
        assert_eq!(1, calc_interval(10));
        assert_eq!(2, calc_interval(90));
        assert_eq!(20, calc_interval(900));
    }
    #[test]
    fn test_get_size_distribution() {
        assert_eq!((10, 1), get_size_distribution(10, 60));
        assert_eq!((8, 2), get_size_distribution(9, 100));
    }
}
