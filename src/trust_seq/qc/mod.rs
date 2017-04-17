mod basic_stats;
mod quality_counts;
mod per_base_quality_scores;
//mod per_tile_quality_scores;
//mod per_sequence_gc_content;
//mod per_base_sequence_content;
//mod per_sequence_quality_scores;



//mod n_content;
//mod sequence_length_distribution;
//mod over_represented_seqs;
use super::utils::Sequence;

use std::io::Write;
use std::io::Error;
use std::io::ErrorKind;
use std::io;

use serde_json::Value;
use serde_json::map::Map;
use self::basic_stats::BasicStats;
use self::per_base_quality_scores::PerBaseQualityScores;
//use self::per_base_sequence_content::PerBaseSequenceContent;
//use self::per_sequence_quality_scores::PerSequenceQualityScores;

//use self::per_sequence_gc_content::PerSequenceGCContents;
use super::trust_seq::{TrustSeqConfig, TrustSeqErr};

//use self::n_content::NContent;
//use self::sequence_length_distribution::SequenceLengthDistribution;
//use self::over_represented_seqs::OverRepresentedSeqs;

pub fn create_qcmodules<'a>(config: &'a TrustSeqConfig) -> Vec<Box<QCModule + 'a>> {
    let mut modules: Vec<Box<QCModule + 'a>> = Vec::new();
    modules.push(Box::new(BasicStats::new()));
    modules.push(Box::new(PerBaseQualityScores::new(config)));
    //    modules.push(Box::new(PerSequenceQualityScores::new()));
    //    modules.push(Box::new(PerBaseSequenceContent::new()));
    //    modules.push(Box::new(PerSequenceGCContents::new()));
    //    modules.push(Box::new(NContent::new()));
    //    modules.push(Box::new(SequenceLengthDistribution::new()));
    //    modules.push(Box::new(OverRepresentedSeqs::new()));
    return modules;
}
pub trait QCModule {
    fn get_name(&self) -> &'static str;
    fn ignore_in_report(&self) -> bool {
        return false;
    }
    fn is_error(&self) -> bool {
        return false;
    }
    fn is_warn(&self) -> bool {
        return false;
    }
    fn get_status(&self) -> &str {
        if self.is_error() {
            return "fail";
        } else if self.is_warn() {
            return "warn";
        } else {
            return "pass";
        }
    }
    fn process_sequence(&mut self, seq: &Sequence) -> ();
    fn print_text_report(&self, w: &mut Write) -> Result<(), TrustSeqErr>;
    fn to_json(&self) -> Result<Value, TrustSeqErr>;
}
pub fn write_text_reports<'a>(modules: &Vec<Box<QCModule + 'a>>,
                              w: &mut Write)
                              -> Result<(), TrustSeqErr> {
    for module in modules {
        writeln!(w, ">>{}\t{}", module.get_name(), module.get_status())?;
        module.print_text_report(w)?;
        writeln!(w, ">>END_MODULE")?;
    }
    return Ok(());
}
pub fn get_json_reports<'a>(modules: &Vec<Box<QCModule + 'a>>)
                            -> Result<Map<String, Value>, TrustSeqErr> {
    let mut map: Map<String, Value> = Map::new();
    for module in modules {
        let report = module.to_json()?;
        map.insert(module.get_name().to_string(), report);
    }
    return Ok(map);
}
#[derive(Clone,Debug)]
pub struct PhreadEncoding {
    pub name: &'static str,
    pub offset: u8,
}
static SANGER_ENCODING_OFFSET: u8 = 33;
static ILUMINA_1_3_ENCODING_OFFSET: u8 = 64;

impl PhreadEncoding {
    pub fn get_phread_encoding(lowest_char: u8) -> io::Result<PhreadEncoding> {
        if lowest_char < 32 {
            return Err(Error::new(ErrorKind::Other, "No known encodings with chars < 33"));
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
            return Err(Error::new(ErrorKind::Other, "No known encodings with chars > 126"));
        }
    }
}
