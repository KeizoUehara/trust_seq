use std::cmp;

use std::f64;
use std::i32;
use std::str;
use std::str::FromStr;
use std::io::Write;
use std::collections::HashMap;
use serde_json::value::Value;
use serde_json::value;
use trust_seq::utils::Sequence;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::QualityCount;
use trust_seq::group::BaseGroup;

pub struct PerTileQualityScores<'a> {
    ignore_in_report: bool,
    total_count: u64,
    id_position: i32,
    min_char: u8,
    max_char: u8,
    quality_counts: HashMap<u32, Vec<QualityCount>>,
    config: &'a TrustSeqConfig,
}

impl<'a> PerTileQualityScores<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerTileQualityScores {
        return PerTileQualityScores {
                   ignore_in_report: false,
                   id_position: -1,
                   quality_counts: HashMap::new(),
                   min_char: 255,
                   max_char: 0,
                   config: config,
               };
    }
}
impl<'a> QCModule for PerTileQualityScores<'a> {
    fn get_name(&self) -> &'static str {
        return "Per tile sequence quality";
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        if self.ignore_in_report {
            return;
        }
        self.total_count += 1;
        let mut tile: i32 = 0;
        let id_str = str::from_utf8_unchecked(seq.id);
        let split_ids: Vec<&str> = id_str.split(":").collect();
        if self.id_position < 0 {
            if split_ids.len() >= 7 {
                self.id_position = 4;
            } else if split_ids.len() >= 5 {
                self.id_position = 2;
            } else {
                self.ignore_in_report = true;
                return;
            }
        }
        match i32::from_str(split_ids[self.id_position as usize]) {
            Ok(v) => tile = v,
            Err(e) => {
                self.ignore_in_report = true;
                return;
            }
        };
        if !self.quality_counts.contains_key(&(tile as u32)) {
            if self.quality_counts.len() > 500{
                println!("Too many tiles (>500) so giving up trying to do per-tile qualities since we're probably parsing the file wrongly");
                self.ignore_in_report = true;
                return;
            }
        }
        let len = seq.quality.len();
        if self.quality_counts.len() < len {
            self.quality_counts.resize(len, QualityCount::new());
        }
        for (idx, ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char, *ch);
            self.max_char = cmp::max(self.max_char, *ch);
            self.quality_counts[idx].add_value(*ch as usize);
        }
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        let report = self.get_qualitys()?;
        return Ok(value::to_value(&report)?);
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        let vals = self.get_qualitys()?;
        for q in vals.quality_data {
            if q.lower_base == q.upper_base {
                write!(writer,
                       "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                       q.lower_base,
                       q.mean,
                       q.median,
                       q.lower_quartile,
                       q.upper_quartile,
                       q.percentile_10,
                       q.percentile_90)?;
            } else {
                write!(writer,
                       "{}-{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                       q.lower_base,
                       q.upper_base,
                       q.mean,
                       q.median,
                       q.lower_quartile,
                       q.upper_quartile,
                       q.percentile_10,
                       q.percentile_90)?;
            }
        }
        return Ok(());
    }
}
