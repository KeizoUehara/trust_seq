use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;

pub struct NContent {
    n_counts: Vec<u64>,
    not_n_counts: Vec<u64>,
}
impl NContent {
    pub fn new() -> NContent {
        return NContent {
                   n_counts: Vec::new(),
                   not_n_counts: Vec::new(),
               };
    }
}

impl QCModule for NContent {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        writeln!(writer, "#Base\tN-Count");
        for idx in 0..self.n_counts.len() {
            let n_count = self.n_counts[idx] as f64;
            let percentage: f64 = 100.0 * n_count / (n_count + self.not_n_counts[idx] as f64);
            try!(writeln!(writer, "{}\t{}", idx + 1, percentage));
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
