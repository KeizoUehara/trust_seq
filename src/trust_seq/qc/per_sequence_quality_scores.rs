use std::io::Write;
use serde_json::value::Value;
use serde_json::value;
use serde_json::map::Map;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;
use trust_seq::qc::{QCModule, QCResult, QCReport};
use trust_seq::qc::PhreadEncoding;

pub struct PerSequenceQualityScores<'a> {
    score_counts: [u64; 128],
    lowest_char: u8,
    config: &'a TrustSeqConfig,
}
#[derive(Serialize)]
struct PerSequenceQualityReport {
    status: QCResult,
    qualities: Vec<(u32, u64)>,
}
impl<'a> PerSequenceQualityScores<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerSequenceQualityScores {
        return PerSequenceQualityScores {
                   score_counts: [0; 128],
                   lowest_char: 255,
                   config: config,
               };
    }
}
impl QCReport for PerSequenceQualityReport {
    fn get_name(&self) -> &'static str {
        return "Per sequence quality scores";
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(&self)?);
        return Ok(());
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        for val in &self.qualities {
            write!(writer, "{}\t{}\n", val.0, val.1)?;
        }
        return Ok(());
    }
}
impl<'a> QCModule for PerSequenceQualityScores<'a> {
    fn calculate(&self, reports: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        let mut min_score = 128;
        let mut max_score = 0;
        for (score, count) in self.score_counts.iter().enumerate() {
            if *count > 0 {
                if min_score == 128 {
                    min_score = score;
                }
                max_score = score;
            }
        }
        let mut qualities: Vec<(u32, u64)> = Vec::new();
        let mut max_count = 0;
        let mut most_frequence_score = 0.0;
        let encoding = PhreadEncoding::get_phread_encoding(self.lowest_char)?;
        for score in min_score..(max_score + 1) {
            qualities.push((score as u32 - (encoding.offset as u32), self.score_counts[score]));
            if max_count < self.score_counts[score] {
                max_count = self.score_counts[score];
                most_frequence_score = score as f64;
            }

        }
        let error_th = self.config.module_config.get("quality_sequence:error");
        let warn_th = self.config.module_config.get("quality_sequence:warn");
        let status = if most_frequence_score > error_th {
            QCResult::Fail
        } else if most_frequence_score > warn_th {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        reports.push(Box::new(PerSequenceQualityReport {
                                  status: status,
                                  qualities: qualities,
                              }));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {

        let mut average_quality: usize = 0;
        for ch in seq.quality {
            if *ch < self.lowest_char {
                self.lowest_char = *ch;
            }
            average_quality += *ch as usize;
        }
        if 0 < seq.quality.len() {
            average_quality /= seq.quality.len();
            self.score_counts[average_quality] += 1;
        }
    }
}
