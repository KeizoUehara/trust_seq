use std::cmp;
use std::f64;
use std::io::Write;
use serde_json::value::Value;
use serde_json::value;
use trust_seq::utils::Sequence;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::qc::QCModule;
use trust_seq::qc::QCResult;
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::quality_counts::QualityCounts;
use trust_seq::group::BaseGroup;

pub struct PerBaseQualityScores<'a> {
    min_char: u8,
    max_char: u8,
    qualities: QualityCounts,
    config: &'a TrustSeqConfig,
    report: Option<PerBaseQualityReport>,
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
                   report: None,
               };
    }
}
impl<'a> QCModule for PerBaseQualityScores<'a> {
    fn get_name(&self) -> &'static str {
        return "Per base sequence quality";
    }
    fn calculate(&mut self) -> Result<(), TrustSeqErr> {
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
        let lower_error_th = self.config
            .module_config
            .get("quality_base_lower:error");
        let median_error_th = self.config
            .module_config
            .get("quality_base_median:error");
        let lower_warn_th = self.config.module_config.get("quality_base_lower:warn");
        let median_warn_th = self.config
            .module_config
            .get("quality_base_median:warn");
        let status = if min_median < median_error_th || min_quartile < lower_error_th {
            QCResult::fail
        } else if min_median < median_warn_th || min_quartile < lower_warn_th {
            QCResult::warn
        } else {
            QCResult::pass
        };
        self.report = Some(PerBaseQualityReport {
                               status: status,
                               quality_data: v,
                           });
        return Ok(());
    }
    fn get_status(&self) -> QCResult {
        return self.report.as_ref().unwrap().status;
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
        let report = self.report.as_ref().unwrap();
        return Ok(value::to_value(&report)?);
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        let vals = self.report.as_ref().unwrap();
        for q in &vals.quality_data {
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
