use crate::trust_seq::gc_model::GCModel;
use crate::trust_seq::qc::{QCModule, QCReport, QCResult};
use crate::trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use crate::trust_seq::utils::Sequence;
use serde_json::map::Map;
use serde_json::value;
use serde_json::value::Value;
use std::collections::hash_map::HashMap;
use std::f64;
use std::io::Write;
use std::slice::Iter;

pub struct PerSequenceGCContents<'a> {
    config: &'a TrustSeqConfig,
    gc_distribution: [f64; 101],
    gc_models: HashMap<usize, Box<GCModel>>,
}
#[derive(Serialize)]
struct PerSequenceGCReport {
    status: QCResult,
    gc_distribution: Vec<f64>,
    theoretical_distribution: Vec<f64>,
}
impl<'a> PerSequenceGCContents<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerSequenceGCContents {
        let mut gc_distribution = Vec::new();
        gc_distribution.resize(101, 0.0);
        return PerSequenceGCContents {
            config: config,
            gc_distribution: [0f64; 101],
            gc_models: HashMap::new(),
        };
    }
}
fn truncate_sequence(sequence: &[u8]) -> &[u8] {
    if sequence.len() > 1000 {
        let length = (sequence.len() / 1000) * 1000;
        return &sequence[..length];
    } else if sequence.len() > 100 {
        let length = (sequence.len() / 100) * 100;
        return &sequence[..length];
    } else {
        return sequence;
    }
}
fn calc_zscore_for_value(mean: f64, stddev: f64, value: f64) -> f64 {
    let lhs = (2f64 * f64::consts::PI * stddev * stddev).sqrt();
    let rhs = f64::consts::E.powf(-1.0 * (value - mean).powi(2) / (2.0 * stddev * stddev));
    rhs / lhs
}
fn calc_stddev_total(values: Iter<f64>, mode: f64) -> (f64, f64) {
    let mut total_count = 0.0;
    let mut stddev = 0.0;
    for (i, v) in values.enumerate() {
        stddev += (i as f64 - mode).powi(2) * (*v);
        total_count += *v;
    }
    stddev /= total_count - 1f64;
    (stddev.sqrt(), total_count)
}
impl QCReport for PerSequenceGCReport {
    fn get_name(&self) -> &'static str {
        return "Per sequence GC content";
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        writeln!(writer, "#GC Content\tCount")?;
        for idx in 0..101 {
            writeln!(writer, "{}\t{}", idx, self.gc_distribution[idx])?;
        }
        return Ok(());
    }
}
impl<'a> QCModule for PerSequenceGCContents<'a> {
    fn calculate(&self, reports: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        let mode = self
            .gc_distribution
            .iter()
            .enumerate()
            .max_by(|a, b| (a.1).partial_cmp(b.1).unwrap())
            .unwrap();
        let mode_th = mode.1 * 0.90;
        let mut mode_total = 0;
        let mut mode_count: u32 = 0;
        let mut fell_off_top = true;
        let mut fell_off_bottom = true;
        for idx in (mode.0)..self.gc_distribution.len() {
            if self.gc_distribution[idx] > mode_th {
                mode_total += idx;
                mode_count += 1;
            } else {
                fell_off_top = false;
                break;
            }
        }
        for idx in (0..mode.0).rev() {
            if self.gc_distribution[idx] > mode_th {
                mode_total += idx;
                mode_count += 1;
            } else {
                fell_off_bottom = false;
                break;
            }
        }
        let mode2: f64 = if fell_off_top || fell_off_bottom {
            *mode.1
        } else {
            mode_total as f64 / mode_count as f64
        };
        let (stddev, total_count) = calc_stddev_total(self.gc_distribution.iter(), mode2);
        let mut theoretical_distribution = [0.0 as f64; 101];
        let mut deviation_percent = 0.0;
        for (i, v) in theoretical_distribution.iter_mut().enumerate() {
            *v = calc_zscore_for_value(mode2, stddev, i as f64) * total_count;
            deviation_percent += (*v - self.gc_distribution[i]).abs();
        }
        deviation_percent = deviation_percent * 100.0 / total_count;
        let error_th = self.config.module_config.get("gc_sequence:error");
        let warn_th = self.config.module_config.get("gc_sequence:warn");

        let status = if deviation_percent > error_th {
            QCResult::Fail
        } else if deviation_percent > warn_th {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        reports.push(Box::new(PerSequenceGCReport {
            status: status,
            gc_distribution: self.gc_distribution.to_vec(),
            theoretical_distribution: theoretical_distribution.to_vec(),
        }));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let mut gc_count: usize = 0;
        let sequence = truncate_sequence(seq.sequence);
        for s in sequence {
            let ch = *s as char;
            let is_gc = match ch {
                'G' => true,
                'g' => true,
                'c' => true,
                'C' => true,
                _ => false,
            };
            if is_gc {
                gc_count += 1;
            }
        }
        let seq_len = seq.sequence.len();
        if !self.gc_models.contains_key(&seq_len) {
            self.gc_models
                .insert(seq_len, Box::new(GCModel::new(seq_len)));
        }
        match self.gc_models.get(&seq_len) {
            Some(model) => model.add_value(gc_count, &mut self.gc_distribution),
            None => (),
        }
    }
}
