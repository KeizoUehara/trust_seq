use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;

pub struct SequenceLengthDistribution {
    length_counts: Vec<u64>,
}
impl NContent {
    pub fn new() -> NContent {
        return NContent { length_counts: Vec::new() };
    }
}
fn get_min_max_idx(vec: &Vec<u64>) -> (usize, usize) {
    let mut min: i32 = -1;
    let mut max: usize = 0;
    for (idx, val) in vec.iter().enumerate() {
        if val > 0 {
            if min < 0 {
                min = idx;
            }
            max = idx;
        }
    }
    return (min as usize, max);
}
fn calc_interval(width: usize) -> usize {
    let interval = 1;
    let divisions = [1,2,5];
    loop {
        for division in divitions{
            if width / interval * division <= 50{
                return interval * division;
            }
        }
        interval *= 10;
    }
}
fn get_size_distribution(min:usize,max:usize) -> (usize,usize) {
    return (0,0);
}
impl QCModule for SequenceLengthDistribution {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        let (mut min_len, mut max_len) = get_min_max_idx(self.length_counts);
        if min_len > 0 {
            min_len--;
        }
        max_len++;
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
        if self.length_counts.len() < len + 2 {
            self.length_counts.resize(len, 0);
        }
        self.length_counts[len] += 1;
    }
}
#[cfg(test)}
mod tests {
    use super::*;

    #[test]
    fn test_calc_interval(){
        assert_eq!(1,calc_interval(10));
        assert_eq!(2,calc_interval(90));
    }
}
