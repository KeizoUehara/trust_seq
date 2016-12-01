use std::io::Write;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use trust_seq::per_base_sequence_content::PerBaseSequenceContent;
use trust_seq::per_sequence_quality_scores::PerSequenceQualityScores;
use trust_seq::per_base_quality_scores::PerBaseQualityScores;
use trust_seq::basic_stats::BasicStats;
use trust_seq::utils::Sequence;

pub fn create_qcmodules() -> Vec<Box<QCModule>> {
    let mut modules :Vec<Box<QCModule>> = Vec::new();
    modules.push(Box::new(BasicStats::new()));
    modules.push(Box::new(PerBaseQualityScores::new()));
    modules.push(Box::new(PerSequenceQualityScores::new()));
    modules.push(Box::new(PerBaseSequenceContent::new()));
    return modules;
}
pub trait QCModule{    
    fn process_sequence(&mut self,seq:&Sequence) -> ();
    fn print_report(&mut self) -> ();
    fn print_text_report(&self,w:&mut Write) -> Result<()>;
}
#[derive(Copy)]
pub struct QualityCount {
    pub counts: [u64;150],
}
impl QualityCount {
    pub fn new() -> QualityCount {
        return QualityCount { counts: [0;150]}; 
    }
    pub fn get_mean(&self,offset :u32) -> f64 {
        let mut total: f64 = 0.0;
        let mut count: f64 = 0.0;
        for (idx,c) in self.counts.iter().enumerate() {
            let cnt = *c as f64;
            total += cnt * ((idx as f64) - (offset as f64));
            count += cnt;
        }
        return total / count;
    }
    pub fn get_percentile(&self,offset :u32, percentile:u32) -> u32{
        let mut total :u64 = 0;
        for c in self.counts.iter(){
            total += *c;
        }
        total = total * (percentile as u64)/ 100;
        let mut count = 0;
        for (i,c) in self.counts.iter().enumerate() {
            count += *c;
            if count >= total {
                return (i as u32 - offset) as u32;
            }
        }
        return 0;
    }
}
impl Clone for QualityCount{
    fn clone(&self) -> QualityCount{
        return *self;
    }
}
#[derive(Clone,Debug)]
pub struct PhreadEncoding {
    pub name: &'static str,
    pub offset: u8
}
static SANGER_ENCODING_OFFSET:u8 = 33;
static ILUMINA_1_3_ENCODING_OFFSET:u8 = 64;

impl PhreadEncoding {
    pub fn get_phread_encoding(lowest_char:u8) -> Result<PhreadEncoding> {
        if lowest_char < 32 {
            return Err(Error::new(ErrorKind::Other,
                                  "No known encodings with chars < 33"));
        }else if lowest_char < 64{
            return Ok(PhreadEncoding{
                offset: SANGER_ENCODING_OFFSET,
                name:  "Sanger / Illumina 1.9"
            });
        }else if lowest_char == ILUMINA_1_3_ENCODING_OFFSET + 1 {
            return Ok(PhreadEncoding{
                offset: ILUMINA_1_3_ENCODING_OFFSET ,
                name:  "Illumina 1.3"
            });
        }else if lowest_char <= 126 {
            return Ok(PhreadEncoding{
                offset: ILUMINA_1_3_ENCODING_OFFSET ,
                name:  "Illumina 1.5"
            });
        }else{
            return Err(Error::new(ErrorKind::Other,
                                  "No known encodings with chars > 126"));
        }
    }
    
}
