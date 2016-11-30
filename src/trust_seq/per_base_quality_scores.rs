use std::cmp;
use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::QualityCount;

pub struct PerBaseQualityScores{
    min_char :u8,
    max_char :u8,
    quality_counts: Vec<QualityCount>
}
impl PerBaseQualityScores{
    pub fn new() -> PerBaseQualityScores {
        return PerBaseQualityScores {
            quality_counts: Vec::new(),
            min_char: 255,
            max_char: 0
        };
    }
}
impl QCModule for PerBaseQualityScores {
    fn print_report(&mut self) -> () {
        for ch in &self.quality_counts {
            println!("mean = {}",ch.get_mean(self.min_char as u32));
        }
    }
    fn process_sequence(&mut self,seq:&Sequence) -> (){
        let len = seq.quality.len();
        if self.quality_counts.len() < len {
            self.quality_counts.resize(len,QualityCount::new());
        }
        for (idx,ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char,*ch);
            self.max_char = cmp::max(self.max_char,*ch);            
            self.quality_counts[idx].counts[*ch as usize] += 1;
        }
    }
    fn print_text_report(&self,writer:&mut Write) -> Result<()> {
        let encode = try!(PhreadEncoding::get_phread_encoding(self.min_char));
        let m = encode.offset as u32;
        for (idx,ch) in self.quality_counts.iter().enumerate() {
            try!(write!(writer,"{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        idx+1,
                        ch.get_mean(m),
                        ch.get_percentile(m,50),
                        ch.get_percentile(m,25),
                        ch.get_percentile(m,75),
                        ch.get_percentile(m,10),
                        ch.get_percentile(m,90)));
        }
        return Ok(());
    }
}
