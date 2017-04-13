use std::cmp;
use std::io::Write;
use serde_json::value::Value;
use serde_json::value;
use trust_seq::module_config::ModuleConfig;
use trust_seq::utils::Sequence;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::QualityCount;

pub struct PerBaseQualityScores<'a> {
    min_char: u8,
    max_char: u8,
    quality_counts: Vec<QualityCount>,
    module_config: &'a ModuleConfig,
}
#[derive(Serialize)]
struct PerBaseQualityReport {
    status: String,
    quality_data: Vec<Quality>,
}
#[derive(Serialize)]
struct Quality {
    base: usize,
    mean: f64,
    median: u32,
    lower_quartile: u32,
    upper_quartile: u32,
    percentile_10: u32,
    percentile_90: u32,
}

impl<'a> PerBaseQualityScores<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerBaseQualityScores {
        return PerBaseQualityScores {
                   quality_counts: Vec::new(),
                   min_char: 255,
                   max_char: 0,
                   module_config: &config.module_config,
               };
    }
    fn get_qualitys(&self) -> Result<PerBaseQualityReport, TrustSeqErr> {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char)?;
        let offset = encode.offset as u32;
        let mut v = Vec::new();
        for (idx, ch) in self.quality_counts.iter().enumerate() {
            v.push(Quality {
                       base: idx + 1,
                       mean: ch.get_mean(offset),
                       median: ch.get_percentile(offset, 50),
                       lower_quartile: ch.get_percentile(offset, 25),
                       upper_quartile: ch.get_percentile(offset, 75),
                       percentile_10: ch.get_percentile(offset, 10),
                       percentile_90: ch.get_percentile(offset, 90),
                   });
        }
        return Ok(PerBaseQualityReport {
                      status: self.get_status().to_string(),
                      quality_data: v,
                  });
    }
}
impl<'a> QCModule for PerBaseQualityScores<'a> {
    fn get_name(&self) -> &'static str {
        return "Per base sequence quality";
    }
    fn is_error(&self) -> bool {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char).unwrap();
        let offset = encode.offset as u32;
        let lower_th = self.module_config.get("quality_base_lower:error") as u32;
        let median_th = self.module_config.get("quality_base_median:error") as u32;
        for ch in &self.quality_counts {
            if ch.get_percentile(offset, 10) < lower_th ||
               ch.get_percentile(offset, 50) < median_th {
                return true;
            }
        }
        return false;
    }
    fn is_warn(&self) -> bool {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char).unwrap();
        let offset = encode.offset as u32;
        let lower_th = self.module_config.get("quality_base_lower:warn") as u32;
        let median_th = self.module_config.get("quality_base_median:warn") as u32;
        for ch in &self.quality_counts {
            if ch.get_percentile(offset, 10) < lower_th ||
               ch.get_percentile(offset, 50) < median_th {
                return true;
            }
        }
        return false;
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let len = seq.quality.len();
        if self.quality_counts.len() < len {
            self.quality_counts.resize(len, QualityCount::new());
        }
        for (idx, ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char, *ch);
            self.max_char = cmp::max(self.max_char, *ch);
            self.quality_counts[idx].counts[*ch as usize] += 1;
        }
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        let report = self.get_qualitys()?;
        return Ok(value::to_value(&report)?);
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        let vals = self.get_qualitys()?;
        for q in vals.quality_data {
            write!(writer,
                   "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                   q.base,
                   q.mean,
                   q.median,
                   q.lower_quartile,
                   q.upper_quartile,
                   q.percentile_10,
                   q.percentile_90)?;
        }
        return Ok(());
    }
}
