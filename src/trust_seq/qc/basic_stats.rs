use std::cmp;
use std::io::Write;
use serde_json::Value;
use serde_json::value;
use serde_json::map::Map;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;
use trust_seq::qc::{QCModule, QCResult};
use trust_seq::qc::PhreadEncoding;
use std::collections::BTreeMap;

pub struct BasicStats {
    actual_count: u64,
    filtered_count: u64,
    min_length: u32,
    max_length: u32,
    lowest_char: u8,
    gatcn_count: [u64; 5],
    report: Option<BasicStatsReport>,
}

impl BasicStats {
    pub fn new() -> BasicStats {
        return BasicStats {
                   actual_count: 0,
                   filtered_count: 0,
                   min_length: 0,
                   max_length: 0,
                   lowest_char: 255,
                   gatcn_count: [0; 5],
                   report: None,
               };
    }
}

#[derive(Serialize)]
struct BasicStatsReport {
    status: QCResult,
    encoding: String,
    total_sequence: u64,
    filtered_sequence: u64,
    sequence_min_length: u32,
    sequence_max_length: u32,
    gc_percent: u32,
}
impl QCModule for BasicStats {
    fn get_name(&self) -> &'static str {
        return "Basic Statistics";
    }
    fn calculate(&mut self) -> Result<(), TrustSeqErr> {
        let encoding = PhreadEncoding::get_phread_encoding(self.lowest_char)?;
        let gc_count = self.gatcn_count[0] + self.gatcn_count[3];
        let at_count = self.gatcn_count[1] + self.gatcn_count[2];
        self.report = Some(BasicStatsReport {
                               status: self.get_status(),
                               encoding: encoding.name.to_string(),
                               total_sequence: self.actual_count,
                               filtered_sequence: self.filtered_count,
                               sequence_min_length: self.max_length,
                               sequence_max_length: self.min_length,
                               gc_percent: ((gc_count * 100) / (gc_count + at_count)) as u32,
                           });
        return Ok(());
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        return return Ok(value::to_value(self.report.as_ref().unwrap())?);
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        let j = self.report.as_ref().unwrap();
        write!(writer, "Encoding\t{}\n", j.encoding)?;
        write!(writer, "Total Sequences\t{}\n", j.total_sequence)?;
        write!(writer, "Filtered Sequences\t{}\n", j.filtered_sequence)?;
        if j.sequence_min_length == j.sequence_max_length {
            write!(writer, "Sequence length\t{}\n", j.sequence_min_length)?;
        } else {
            write!(writer,
                   "Sequence length\t{}-{}\n",
                   j.sequence_min_length,
                   j.sequence_max_length)?;
        }
        write!(writer, "%GC\t{}\n", j.gc_percent)?;
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        self.actual_count += 1;
        let len = seq.sequence.len() as u32;
        if self.actual_count == 1 {
            self.min_length = len;
            self.max_length = len;
        } else {
            self.min_length = cmp::min(self.min_length, len);
            self.max_length = cmp::max(self.min_length, len);
        }
        for q in seq.sequence {
            let ch = *q as char;
            let idx = match ch {
                'G' => 0,
                'A' => 1,
                'T' => 2,
                'C' => 3,
                'g' => 0,
                'a' => 1,
                't' => 2,
                'c' => 3,
                'N' => 4,
                _ => {
                    println!("unexpected char={}", ch);
                    4
                }
            };
            self.gatcn_count[idx] += 1;
        }
        for q in seq.quality {
            if *q < self.lowest_char {
                self.lowest_char = *q;
            }
        }
    }
}
