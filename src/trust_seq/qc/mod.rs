mod adapter_content;
mod basic_stats;
mod kmer_content;
mod n_content;
mod over_represented_seqs;
mod per_base_quality_scores;
mod per_base_sequence_content;
mod per_sequence_gc_content;
mod per_sequence_quality_scores;
mod per_tile_quality_scores;
mod quality_counts;
mod sequence_length_distribution;
use super::utils::Sequence;

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

use self::adapter_content::AdapterContent;
use self::basic_stats::BasicStats;
use self::kmer_content::KmerContent;
use self::n_content::NContent;
use self::over_represented_seqs::OverRepresentedSeqs;
use self::per_base_quality_scores::PerBaseQualityScores;
use self::per_base_sequence_content::PerBaseSequenceContent;
use self::per_sequence_gc_content::PerSequenceGCContents;
use self::per_sequence_quality_scores::PerSequenceQualityScores;
use self::per_tile_quality_scores::PerTileQualityScores;
use self::sequence_length_distribution::SequenceLengthDistribution;
use super::trust_seq::{TrustSeqConfig, TrustSeqErr};
use serde_json::map::Map;
use serde_json::Value;

pub fn create_qcmodules<'a>(config: &'a TrustSeqConfig) -> Vec<Box<QCModule + 'a>> {
    let mut modules: Vec<Box<QCModule + 'a>> = Vec::new();
    modules.push(Box::new(BasicStats::new()));
    modules.push(Box::new(PerBaseQualityScores::new(config)));
    modules.push(Box::new(PerTileQualityScores::new(config)));
    modules.push(Box::new(PerSequenceQualityScores::new(config)));
    modules.push(Box::new(PerBaseSequenceContent::new(config)));
    modules.push(Box::new(PerSequenceGCContents::new(config)));
    modules.push(Box::new(NContent::new(config)));
    modules.push(Box::new(SequenceLengthDistribution::new(config)));
    modules.push(Box::new(OverRepresentedSeqs::new(config)));
    modules.push(Box::new(AdapterContent::new(config)));
    modules.push(Box::new(KmerContent::new(config)));
    return modules;
}
#[derive(Serialize, Debug, Copy, Clone)]
pub enum QCResult {
    Pass,
    Warn,
    Fail,
}
pub trait QCModule {
    fn ignore_in_report(&self) -> bool {
        return false;
    }
    fn process_sequence(&mut self, seq: &Sequence) -> ();
    fn calculate(&self, results: &mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr>;
}

pub trait QCReport {
    fn get_status(&self) -> QCResult;
    fn get_name(&self) -> &'static str;
    fn print_text_report(&self, w: &mut Write) -> Result<(), TrustSeqErr>;
    fn add_json(&self, map: &mut Map<String, Value>) -> Result<(), TrustSeqErr>;
}
pub fn write_text_reports<'a>(
    modules: &Vec<Box<QCModule + 'a>>,
    w: &mut Write,
) -> Result<(), TrustSeqErr> {
    let mut reports: Vec<Box<QCReport>> = Vec::new();
    for module in modules {
        module.calculate(&mut reports)?;
    }
    for report in reports {
        writeln!(w, ">>{}\t{:?}", report.get_name(), report.get_status())?;
        report.print_text_report(w)?;
        writeln!(w, ">>END_MODULE")?;
    }
    return Ok(());
}
pub fn get_json_reports<'a>(
    modules: &Vec<Box<QCModule + 'a>>,
) -> Result<Map<String, Value>, TrustSeqErr> {
    let mut reports: Vec<Box<QCReport>> = Vec::new();
    let mut map: Map<String, Value> = Map::new();
    for module in modules {
        module.calculate(&mut reports)?;
    }
    for report in reports {
        report.add_json(&mut map)?;
    }
    return Ok(map);
}
#[derive(Clone, Debug)]
pub struct PhreadEncoding {
    pub name: &'static str,
    pub offset: u8,
}
static SANGER_ENCODING_OFFSET: u8 = 33;
static ILUMINA_1_3_ENCODING_OFFSET: u8 = 64;

impl PhreadEncoding {
    pub fn get_phread_encoding(lowest_char: u8) -> io::Result<PhreadEncoding> {
        if lowest_char < 32 {
            return Err(Error::new(
                ErrorKind::Other,
                "No known encodings with chars < 33",
            ));
        } else if lowest_char < 64 {
            return Ok(PhreadEncoding {
                offset: SANGER_ENCODING_OFFSET,
                name: "Sanger / Illumina 1.9",
            });
        } else if lowest_char == ILUMINA_1_3_ENCODING_OFFSET + 1 {
            return Ok(PhreadEncoding {
                offset: ILUMINA_1_3_ENCODING_OFFSET,
                name: "Illumina 1.3",
            });
        } else if lowest_char <= 126 {
            return Ok(PhreadEncoding {
                offset: ILUMINA_1_3_ENCODING_OFFSET,
                name: "Illumina 1.5",
            });
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "No known encodings with chars > 126",
            ));
        }
    }
}
