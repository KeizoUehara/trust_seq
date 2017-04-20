use std::io::Write;
use serde_json::value;
use serde_json::value::Value;
use trust_seq::group::BaseGroup;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;
use trust_seq::qc::{QCModule, QCResult};

pub struct NContent<'a> {
    config: &'a TrustSeqConfig,
    n_counts: Vec<u64>,
    not_n_counts: Vec<u64>,
    report: Option<NContentReport>,
}
#[derive(Serialize)]
struct NContentReport {
    status: QCResult,
    groups: Vec<BaseGroup>,
    percentages: Vec<f64>,
}
impl<'a> NContent<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> NContent {
        return NContent {
                   config: config,
                   n_counts: Vec::new(),
                   not_n_counts: Vec::new(),
                   report: None,
               };
    }
}

impl<'a> QCModule for NContent<'a> {
    fn get_name(&self) -> &'static str {
        "Per base N content"
    }
    fn calculate(&mut self) -> Result<(), TrustSeqErr> {
        let mut percentages: Vec<f64> = Vec::new();
        let mut max_percentage: f64 = 0.0;
        let groups = BaseGroup::make_base_groups(&self.config.group_type, self.n_counts.len());

        for group in &groups {
            let mut n_count: f64 = 0.0;
            let mut total_count: f64 = 0.0;
            for idx in (group.lower_count - 1)..group.upper_count {
                n_count += self.n_counts[idx] as f64;
                total_count += (self.n_counts[idx] + self.not_n_counts[idx]) as f64;
            }
            let percant = 100.0 * n_count / total_count;
            percentages.push(percant);
            max_percentage = max_percentage.max(percant);
        }
        let error_th = self.config.module_config.get("n_content:error");
        let warn_th = self.config.module_config.get("n_content:warn");
        let status = if max_percentage > error_th {
            QCResult::fail
        } else if max_percentage > warn_th {
            QCResult::warn
        } else {
            QCResult::pass
        };
        self.report = Some(NContentReport {
                               status: status,
                               groups: groups,
                               percentages: percentages,
                           });
        return Ok(());

    }
    fn get_status(&self) -> QCResult {
        return self.report.as_ref().unwrap().status;
    }
    fn to_json(&self) -> Result<Value, TrustSeqErr> {
        let report = self.report.as_ref().unwrap();
        return Ok(value::to_value(&report)?);
    }

    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        writeln!(writer, "#Base\tN-Count")?;
        let report = self.report.as_ref().unwrap();
        for idx in 0..report.groups.len() {
            let group = &report.groups[idx];
            if group.lower_count == group.upper_count {
                writeln!(writer, "{}\t{}", group.lower_count, report.percentages[idx])?;
            } else {
                writeln!(writer,
                         "{}-{}\t{}",
                         group.lower_count,
                         group.upper_count,
                         report.percentages[idx])?;
            }
        }
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let len = seq.sequence.len();
        if self.n_counts.len() < len {
            self.n_counts.resize(len, 0);
            self.not_n_counts.resize(len, 0);
        }
        for (idx, s) in seq.sequence.iter().enumerate() {
            let ch = *s as char;
            match ch {
                'N' => self.n_counts[idx] += 1,
                _ => self.not_n_counts[idx] += 1,
            };
        }
    }
}
