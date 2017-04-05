use std::io::Write;
use std::io::Result;
use std::cmp;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;

pub struct SequenceLengthDistribution {
    length_counts: Vec<u64>,
}
impl SequenceLengthDistribution {
    pub fn new() -> SequenceLengthDistribution {
        return SequenceLengthDistribution { length_counts: Vec::new() };
    }
}
fn get_min_max_idx(vec: &Vec<u64>) -> (usize, usize) {
    let mut min: i32 = -1;
    let mut max: usize = 0;
    for (idx, val) in vec.iter().enumerate() {
        if *val > 0 {
            if min < 0 {
                min = idx as i32;
            }
            max = idx;
        }
    }
    return (min as usize, max);
}
fn calc_interval(width: usize) -> usize {
    let mut base: usize = 1;
    let divisions = [1, 2, 5];
    loop {
        for division in divisions.iter() {
            let interval = base * division;
            if width / interval <= 50 {
                return interval;
            }
        }
        base *= 10;
    }
}
fn get_size_distribution(min: usize, max: usize) -> (usize, usize) {
    let interval = calc_interval(max - min);
    let base_div = min / interval;
    return (base_div * interval, interval);
}
impl QCModule for SequenceLengthDistribution {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        let (mut min_len, mut max_len) = get_min_max_idx(&self.length_counts);
        //  We put one extra category either side of the actual size
        min_len -= 1;
        max_len += 1;
        let (start, interval) = get_size_distribution(min_len, max_len);
        let mut current_pos = start;

        writeln!(writer, "#Length Count");
        while current_pos <= max_len {
            let max_pos = cmp::min(max_len, current_pos + interval);
            let mut count = 0;
            for idx in current_pos..max_pos {
                if idx < self.length_counts.len() {
                    count += self.length_counts[idx];
                }
            }
            if interval == 1 {
                writeln!(writer, "{}\t{}", current_pos, count);
            } else {
                writeln!(writer, "{}-{}\t{}", current_pos, max_pos, count);
            }
            current_pos += interval;
        }
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let len = seq.sequence.len();
        if self.length_counts.len() < len + 2 {
            self.length_counts.resize(len + 2, 0);
        }
        self.length_counts[len] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_interval() {
        assert_eq!(1, calc_interval(10));
        assert_eq!(2, calc_interval(90));
        assert_eq!(20, calc_interval(900));
    }
    #[test]
    fn test_get_size_distribution() {
        assert_eq!((10, 1), get_size_distribution(10, 60));
        assert_eq!((8, 2), get_size_distribution(9, 100));
    }
}
