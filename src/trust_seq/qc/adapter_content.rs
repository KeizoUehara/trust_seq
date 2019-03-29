use serde_json::map::Map;
use serde_json::value;
use serde_json::value::Value;
use std::cmp;
use std::io::BufReader;
use std::io::Write;
use std::str;
use trust_seq::adapter::Adapter;
use trust_seq::adapter_list::ADAPTER_LIST;
use trust_seq::group::BaseGroup;
use trust_seq::qc::{QCModule, QCReport, QCResult};
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::utils::Sequence;

#[derive(Debug)]
pub struct AdapterContent<'a> {
    config: &'a TrustSeqConfig,
    longest_sequence: usize,
    longest_adapter: usize,
    total_count: u64,
    adapters: Vec<Adapter>,
}
#[derive(Serialize)]
struct AdapterContentReport {
    status: QCResult,
    groups: Vec<BaseGroup>,
    adapter_names: Vec<String>,
    enrichments: Vec<Vec<f64>>,
}
impl QCReport for AdapterContentReport {
    fn get_name(&self) -> &'static str {
        "Adapter Content"
    }
    fn get_status(&self) -> QCResult {
        return self.status;
    }
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(&self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        write!(writer, "#Position")?;
        for name in &self.adapter_names {
            write!(writer, "\t{}", name)?;
        }
        write!(writer, "\n")?;
        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.lower_count == group.upper_count {
                write!(writer, "{}", group.lower_count)?;
            } else {
                write!(writer, "{}-{}", group.lower_count, group.upper_count)?;
            }
            for adapter_idx in 0..self.adapter_names.len() {
                write!(writer, "\t{}", self.enrichments[adapter_idx][group_idx])?;
            }
            write!(writer, "\n")?;
        }
        return Ok(());
    }
}
impl<'a> AdapterContent<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> AdapterContent<'a> {
        let adapters = Adapter::load_adapters(BufReader::new(ADAPTER_LIST.as_bytes()));

        let length = adapters
            .iter()
            .fold(0, |acc, ref a| cmp::max(acc, a.sequence.len()));
        return AdapterContent {
            config: config,
            total_count: 0,
            longest_sequence: 0,
            longest_adapter: length,
            adapters: adapters,
        };
    }
    fn ignore_filtered_sequences(&self) -> bool {
        return true;
    }
}

impl<'a> QCModule for AdapterContent<'a> {
    fn calculate(&self, results: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        let groups = BaseGroup::make_base_groups(&self.config.group_type, self.longest_sequence);
        let mut enrichments: Vec<Vec<f64>> = Vec::new();
        let mut adapter_names: Vec<String> = Vec::new();
        let mut max_enrichment = 0.0f64;
        for adapter in &self.adapters {
            adapter_names.push(adapter.name.clone());
            enrichments.push(vec![0.0; groups.len()]);
        }
        for (a, adapter) in self.adapters.iter().enumerate() {
            for (g, group) in groups.iter().enumerate() {
                for idx in (group.lower_count - 1)..group.upper_count {
                    enrichments[a][g] += adapter.positions[idx] as f64;
                }
                enrichments[a][g] *= 100.0
                    / self.total_count as f64
                    / (group.lower_count - group.upper_count + 1) as f64;
                max_enrichment = max_enrichment.max(enrichments[a][g]);
            }
        }
        let status = if max_enrichment > self.config.module_config.get("adapter:error") {
            QCResult::Fail
        } else if max_enrichment > self.config.module_config.get("adapter:warn") {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        results.push(Box::new(AdapterContentReport {
            status: status,
            groups: groups,
            adapter_names: adapter_names,
            enrichments: enrichments,
        }));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        self.total_count += 1;
        if seq.sequence.len() - self.longest_adapter as usize > self.longest_sequence {
            self.longest_sequence = seq.sequence.len() - self.longest_adapter;
            for adapter in &mut self.adapters {
                adapter.positions.resize(self.longest_sequence, 0 as u64)
            }
        }
        for adapter in &mut self.adapters {
            let seq = unsafe { str::from_utf8_unchecked(seq.sequence) };
            match seq.find(&adapter.sequence) {
                Some(idx) => adapter.increment_count(idx),
                _ => (),
            };
        }
    }
}
