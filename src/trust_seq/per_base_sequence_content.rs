use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;

pub struct PerBaseSequenceContent {
    counts: [Vec<u64>;4],
}
impl PerBaseSequenceContent {
    pub fn new() -> PerBaseSequenceContent{
        return PerBaseSequenceContent {
            counts: [Vec::new(),Vec::new(),Vec::new(),Vec::new()]
        }
    }
}

impl QCModule for PerBaseSequenceContent{
    fn print_report(&mut self) -> () {
    }
    fn print_text_report(&self,writer:&mut Write) -> Result<()> {
        for idx in 0 .. self.counts[0].len() {
            try!(write!(writer,"{}\t{}\t{}\t{}\t{}\n",
                        (idx+1),
                        self.counts[0][idx],
                        self.counts[1][idx],
                        self.counts[2][idx],
                        self.counts[3][idx]));
        }
        return Ok(());
    }
    fn process_sequence(&mut self,seq:&Sequence) -> () {
        if self.counts[0].len() < seq.sequence.len() {
            for i in self.counts[0].len() .. seq.sequence.len() {
                for count in &mut self.counts{
                    count.push(0);
                }
            }
        }
        for (idx,ch) in seq.sequence.iter().enumerate() {
            let b = match *ch as char {
                'G'=>0,
                'g'=>0,
                'A'=>1,
                'a'=>1,
                'T'=>2,
                't'=>2,
                'C'=>3,
                'c'=>3,
                _  =>4,
            };
            if b < 4 {
                self.counts[b][idx] += 1;
            }
        }
    }
}
