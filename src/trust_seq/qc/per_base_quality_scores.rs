use std::cmp;
use std::f64;
use std::io::Write;
use serde_json::value::Value;
use serde_json::value;
use trust_seq::utils::Sequence;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::quality_counts::QualityCounts;
use trust_seq::group::BaseGroup;

pub struct PerBaseQualityScores<'a> {
    min_char: u8,
    max_char: u8,
    qualities: QualityCounts,
    config: &'a TrustSeqConfig,
}
#[derive(Serialize)]
struct PerBaseQualityReport {
    status: String,
    quality_data: Vec<Quality>,
}
#[derive(Serialize)]
struct Quality {
    lower_base: usize,
    upper_base: usize,
    mean: f64,
    median: f64,
    lower_quartile: f64,
    upper_quartile: f64,
    percentile_10: f64,
    percentile_90: f64,
}

impl<'a> PerBaseQualityScores<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerBaseQualityScores {
        return PerBaseQualityScores {
                   qualities: QualityCounts::new(),
                   min_char: 255,
                   max_char: 0,
                   config: config,
               };
    }
    fn get_qualitys(&self) -> Result<PerBaseQualityReport, TrustSeqErr> {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char)?;
        let offset = encode.offset as u32;
        let mut v = Vec::new();
        let groups = BaseGroup::make_base_groups(&self.config.group_type, self.qualities.len());
        for (idx, group) in groups.iter().enumerate() {
            v.push(Quality {
                       lower_base: group.lower_count,
                       upper_base: group.upper_count,
                       mean: self.qualities.get_mean(group, offset),
                       median: self.qualities.get_percentile(group, offset, 50),
                       lower_quartile: self.qualities.get_percentile(group, offset, 25),
                       upper_quartile: self.qualities.get_percentile(group, offset, 75),
                       percentile_10: self.qualities.get_percentile(group, offset, 10),
                       percentile_90: self.qualities.get_percentile(group, offset, 90),
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
        let lower_th = self.config
            .module_config
            .get("quality_base_lower:error") as u32;
        let median_th = self.config
            .module_config
            .get("quality_base_median:error") as u32;

        return !(self.qualities.get_min_percentile(offset, 10) < lower_th ||
                 self.qualities.get_min_percentile(offset, 50) < median_th);
    }
    fn is_warn(&self) -> bool {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char).unwrap();
        let offset = encode.offset as u32;
        let lower_th = self.config.module_config.get("quality_base_lower:warn") as u32;
        let median_th = self.config
            .module_config
            .get("quality_base_median:warn") as u32;
        return !(self.qualities.get_min_percentile(offset, 10) < lower_th ||
                 self.qualities.get_min_percentile(offset, 50) < median_th);
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let len = seq.quality.len();
        self.qualities.ensure_size(len);
        for (idx, ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char, *ch);
            self.max_char = cmp::max(self.max_char, *ch);
            self.qualities.add_value(idx, *ch);
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
