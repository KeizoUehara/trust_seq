use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;

pub struct PerSequenceQualityScores {
    score_counts: [u64;128],
    lowest_char:u8,
}
impl PerSequenceQualityScores {
    pub fn new() -> PerSequenceQualityScores{
        return PerSequenceQualityScores {
            score_counts: [0;128],
            lowest_char: 255,
        }
    }
}

impl QCModule for PerSequenceQualityScores{
    fn print_report(&mut self) -> () {
    }
    fn print_text_report(&self,writer:&mut Write) -> Result<()> {
        let mut min_score = 128;
        let mut max_score = 0;
        for (score,count) in self.score_counts.iter().enumerate() {
            if *count > 0 {
                if min_score == 128{
                    min_score = score;
                }
                max_score = score;
            }
        }
        let encoding = try!(PhreadEncoding::get_phread_encoding(self.lowest_char));
        for score in min_score .. (max_score+1) {
            try!(write!(writer,"{}\t{}\n",
                        score - (encoding.offset as usize),
                        self.score_counts[score]
            ));
        }
        return Ok(());
    }
    fn process_sequence(&mut self,seq:&Sequence) -> () {
        let mut average_quality:usize = 0;
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
