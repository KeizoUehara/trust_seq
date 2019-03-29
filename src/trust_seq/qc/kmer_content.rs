use crate::trust_seq::group::BaseGroup;
use crate::trust_seq::math::calc_binomial_distribution_cummulative;
use crate::trust_seq::qc::{QCModule, QCReport, QCResult};
use crate::trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use crate::trust_seq::utils::Sequence;
use serde_json::map::Map;
use serde_json::value;
use serde_json::value::Value;
use std::cmp;
use std::collections::HashMap;
use std::f64;
use std::io::Write;
use std::str;

#[derive(Clone, Debug)]

struct Kmer {
    sequence: String,
    count: u64,
    positions: Vec<u64>,
}

impl Kmer {
    fn new(sequence: &str, position: usize, seq_length: usize) -> Kmer {
        let mut kmer = Kmer {
            sequence: sequence.to_string(),
            count: 1,
            positions: vec![0; seq_length],
        };
        kmer.positions[position] += 1;
        return kmer;
    }
    fn increment_count(&mut self, position: usize) {
        self.count += 1;
        if self.positions.len() <= position {
            self.positions.resize(position, 0);
        }
        self.positions[position] += 1;
    }
}
#[derive(Debug)]
pub struct KmerContent<'a> {
    config: &'a TrustSeqConfig,
    skip_count: u64,
    longest_sequence: usize,
    kmers: HashMap<String, Kmer>,
    total_kmer_counts: Vec<Vec<u64>>,
}
#[derive(Serialize)]
struct KmerContentReport {
    status: QCResult,
    kmers: Vec<KmerReport>,
}
#[derive(Serialize)]
struct KmerReport {
    sequence: String,
    count: u64,
    p_value: f64,
    max_obs_exp: f32,
    max_lower_position: usize,
    max_upper_position: usize,
}
impl QCReport for KmerContentReport {
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
        writeln!(
            writer,
            "#Sequence\tCount\tPValue\tObs/Exp Max\tMax Obs/Exp Position"
        )?;
        for kmer in &self.kmers {
            if kmer.max_lower_position == kmer.max_upper_position {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}",
                    &kmer.sequence,
                    kmer.count,
                    kmer.p_value,
                    kmer.max_obs_exp,
                    kmer.max_lower_position
                )?;
            } else {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}-{}",
                    &kmer.sequence,
                    kmer.count,
                    kmer.p_value,
                    kmer.max_obs_exp,
                    kmer.max_lower_position,
                    kmer.max_upper_position
                )?;
            }
        }
        return Ok(());
    }
}
impl<'a> KmerContent<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> KmerContent<'a> {
        return KmerContent {
            config: config,
            skip_count: 0,
            longest_sequence: 0,
            kmers: HashMap::new(),
            total_kmer_counts: Vec::new(),
        };
    }
    fn ignore_filtered_sequences(&self) -> bool {
        return true;
    }
}
const MIN_KMER_SIZE: usize = 7;
const MAX_KMER_SIZE: usize = 7;

impl<'a> KmerContent<'a> {
    fn add_kmer_count(&mut self, position: usize, kmer_length: usize, kmer: &[u8]) -> bool {
        if position >= self.total_kmer_counts.len() {
            self.total_kmer_counts
                .resize(position + 1, vec![0; MAX_KMER_SIZE]);
        }
        if kmer.iter().any(|&c| c == 'N' as u8) {
            return true;
        }
        // println!("position={},kmer_length={},total_kmer_counts.len()={}",
        //          position,
        //          kmer_length,
        //          self.total_kmer_counts.len());
        self.total_kmer_counts[position][kmer_length - 1] += 1;
        return false;
    }
}

impl<'a> QCModule for KmerContent<'a> {
    fn calculate(&self, results: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr> {
        let groups = BaseGroup::make_base_groups(
            &self.config.group_type,
            self.longest_sequence - MIN_KMER_SIZE + 1,
        );

        let mut uneven_kmers: Vec<KmerReport> = Vec::new();
        for kmer in &mut self.kmers.values() {
            let total_kmer_count = self
                .total_kmer_counts
                .iter()
                .fold(0u64, |acc, ref k| acc + k[kmer.sequence.len() - 1]);
            if kmer.sequence == "GAGCTCA" {
                println!("{} {} {:?}", kmer.sequence, kmer.count, kmer.positions);
            }
            let expected_proportion = kmer.count as f32 / total_kmer_count as f32;
            let mut obs_exp_positions: Vec<f32> = vec![0f32; groups.len()];
            let mut binomial_p_values: Vec<f32> = vec![0f32; groups.len()];
            for (g, group) in groups.iter().enumerate() {
                // This is a summation of the number of Kmers of this length which
                // fall into this base group
                let mut total_group_count = 0u64;

                // This is a summation of the number of hit Kmers which fall within
                // this base group.
                let mut total_group_hits = 0u64;
                let p_max = cmp::min(group.upper_count, kmer.positions.len());
                for p in (group.lower_count - 1)..p_max {
                    total_group_count += self.total_kmer_counts[p][kmer.sequence.len() - 1];
                    total_group_hits += kmer.positions[p];
                }
                let predicted: f32 = expected_proportion as f32 * total_group_count as f32;
                obs_exp_positions[g] = total_group_hits as f32 / predicted;
                if total_group_hits as f32 > predicted {
                    let val =
                        (1.0 - calc_binomial_distribution_cummulative(
                            total_group_count as usize,
                            expected_proportion as f64,
                            total_group_hits as i32,
                        )) * (4.0f64).powi(kmer.sequence.len() as i32);
                    binomial_p_values[g] = val as f32;
                } else {
                    binomial_p_values[g] = 1.0;
                }
            }

            let mut lowest_p_value = 0.01;
            for i in 0..groups.len() {
                if binomial_p_values[i] < lowest_p_value && obs_exp_positions[i] > 5f32 {
                    lowest_p_value = binomial_p_values[i];
                }
            }
            if lowest_p_value < 0.01 {
                let mut max_obs_exp = 0.0;
                let mut max_lower_position = 0usize;
                let mut max_upper_position = 0usize;

                for (idx, val) in obs_exp_positions.iter().enumerate() {
                    if max_obs_exp < *val {
                        max_obs_exp = *val;
                        max_lower_position = groups[idx].lower_count;
                        max_upper_position = groups[idx].upper_count;
                    }
                }
                uneven_kmers.push(KmerReport {
                    sequence: kmer.sequence.clone(),
                    count: kmer.count * 5,
                    p_value: lowest_p_value as f64,
                    max_obs_exp: max_obs_exp,
                    max_lower_position: max_lower_position,
                    max_upper_position: max_upper_position,
                });
            }
        }
        uneven_kmers.sort_by(|a, b| b.max_obs_exp.partial_cmp(&a.max_obs_exp).unwrap());
        let mut final_kmers: Vec<KmerReport> = Vec::new();
        for kmer in uneven_kmers {
            final_kmers.push(kmer);
            if 20 <= final_kmers.len() {
                break;
            }
        }
        let min_p_value = if final_kmers.len() > 0 {
            -1.0 * final_kmers[0].p_value.log10()
        } else {
            1.0
        };
        let status = if min_p_value > self.config.module_config.get("kmer:error") {
            QCResult::Fail
        } else if min_p_value > self.config.module_config.get("kmer:warn") {
            QCResult::Warn
        } else {
            QCResult::Pass
        };
        results.push(Box::new(KmerContentReport {
            status: status,
            kmers: final_kmers,
        }));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        self.skip_count += 1;
        if self.skip_count % 50 != 0 {
            return;
        }
        let mut seq = seq.sequence;
        if seq.len() > 500 {
            seq = &seq[0..500];
        }
        self.longest_sequence = cmp::max(self.longest_sequence, seq.len());
        let kmer_size = 7;
        for i in 0..(seq.len() - kmer_size + 1) {
            let kmer = &seq[i..(i + kmer_size)];
            if self.add_kmer_count(i, kmer_size, kmer) {
                continue;
            }
            let kmer_str = unsafe { str::from_utf8_unchecked(kmer) }.to_string();
            if self.kmers.contains_key(&kmer_str) {
                self.kmers.get_mut(&kmer_str).unwrap().increment_count(i);
            } else {
                self.kmers.insert(
                    kmer_str.to_string(),
                    Kmer::new(&kmer_str, i, seq.len() - kmer_size + 1),
                );
            }
        }
    }
}
