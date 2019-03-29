use crate::trust_seq::group::BaseGroup;
use crate::trust_seq::qc::quality_counts::QualityCounts;
use crate::trust_seq::qc::PhreadEncoding;
use crate::trust_seq::qc::{QCModule, QCReport, QCResult};
use crate::trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use crate::trust_seq::utils::Sequence;
use serde_json::map::Map;
use serde_json::value;
use serde_json::value::Value;
use std::cmp;
use std::f64;
use std::io::Write;

pub struct PerBaseQualityScores<'a> {
    min_char: u8,
    max_char: u8,
    qualities: QualityCounts,
    config: &'a TrustSeqConfig,
}
#[derive(Serialize)]
struct PerBaseQualityReport {
    status: QCResult,
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
}
impl QCReport for PerBaseQualityReport {
    fn get_name(&self) -> &'static str {
        return "Per base sequence quality";
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        for q in &self.quality_data {
            if q.lower_base == q.upper_base {
                write!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    q.lower_base,
                    q.mean,
                    q.median,
                    q.lower_quartile,
                    q.upper_quartile,
                    q.percentile_10,
                    q.percentile_90
                )?;
            } else {
                write!(
                    writer,
                    "{}-{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    q.lower_base,
                    q.upper_base,
                    q.mean,
                    q.median,
                    q.lower_quartile,
                    q.upper_quartile,
                    q.percentile_10,
                    q.percentile_90
                )?;
            }
        }
        return Ok(());
    }
}
impl<'a> QCModule for PerBaseQualityScores<'a> {
    fn calculate(&self, reports: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        let encode = PhreadEncoding::get_phread_encoding(self.min_char)?;
        let offset = encode.offset as u32;
        let mut v = Vec::new();
        let groups = BaseGroup::make_base_groups(&self.config.group_type, self.qualities.len());
        let mut min_quartile: f64 = 1000.0;
        let mut min_median: f64 = 1000.0;
        for group in &groups {
            let lower_quartile = self.qualities.get_percentile(group, offset, 25);
            let median = self.qualities.get_percentile(group, offset, 50);
            v.push(Quality {
                lower_base: group.lower_count,
                upper_base: group.upper_count,
                mean: self.qualities.get_mean(group, offset),
                median: median,
                lower_quartile: lower_quartile,
                upper_quartile: self.qualities.get_percentile(group, offset, 75),
                percentile_10: self.qualities.get_percentile(group, offset, 10),
                percentile_90: self.qualities.get_percentile(group, offset, 90),
            });
            min_median = min_median.min(median);
            min_quartile = min_quartile.min(lower_quartile);
        }
        let lower_error_th = self.config.module_config.get("quality_base_lower:error");
        let median_error_th = self.config.module_config.get("quality_base_median:error");
        let lower_warn_th = self.config.module_config.get("quality_base_lower:warn");
        let median_warn_th = self.config.module_config.get("quality_base_median:warn");
        let status = if min_median < median_error_th || min_quartile < lower_error_th {
            QCResult::Fail
        } else if min_median < median_warn_th || min_quartile < lower_warn_th {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        reports.push(Box::new(PerBaseQualityReport {
            status: status,
            quality_data: v,
        }));
        return Ok(());
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
}
