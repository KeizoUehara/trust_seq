use std::cmp;
use std::f64;
use trust_seq::group::BaseGroup;

pub struct QualityCounts {
    pub counts: Vec<QualityCount>,
}
impl QualityCounts {
    pub fn new() -> QualityCounts {
        return QualityCounts { counts: Vec::new() };
    }
    pub fn ensure_size(&mut self, len: usize) {
        if self.counts.len() < len {
            self.counts.resize(len, QualityCount::new());
        }
    }
    pub fn add_value(&mut self, idx: usize, ch: u8) {
        self.ensure_size(idx + 1);
        self.counts[idx].counts[ch as usize] += 1;
        self.counts[idx].total_count += 1;
    }
    pub fn len(&self) -> usize {
        return self.counts.len();
    }
    pub fn get_mean(&self, group: &BaseGroup, offset: u32) -> f64 {
        let mut count: u32 = 0;
        let mut total: f64 = 0.0;
        for i in (group.lower_count - 1)..group.upper_count {
            count += 1;
            total += self.counts[i].get_mean(offset);
        }
        if count > 0 {
            total / count as f64
        } else {
            f64::NAN
        }
    }
    pub fn get_percentile(&self, group: &BaseGroup, offset: u32, percentile: u32) -> f64 {
        let mut count: u32 = 0;
        let mut total: f64 = 0.0;
        for i in (group.lower_count - 1)..group.upper_count {
            if self.counts[i].total_count > 100 {
                count += 1;
                total += self.counts[i].get_percentile(offset, percentile) as f64;
            }
        }
        if count > 0 {
            total / count as f64
        } else {
            f64::NAN
        }
    }

    pub fn get_min_percentile(&self, offset: u32, percentile: u32) -> u32 {
        let mut min_value: u32 = 1000;
        for c in &self.counts {
            if c.total_count > 100 {
                min_value = cmp::min(min_value, c.get_percentile(offset, percentile));
            }
        }
        min_value
    }
}
#[derive(Copy)]
pub struct QualityCount {
    pub total_count: u64,
    counts: [u64; 150],
}
impl Clone for QualityCount {
    fn clone(&self) -> QualityCount {
        return *self;
    }
}
impl QualityCount {
    pub fn new() -> QualityCount {
        return QualityCount {
            total_count: 0,
            counts: [0; 150],
        };
    }
    pub fn add_value(&mut self, ch: usize) -> () {
        self.counts[ch as usize] += 1;
        self.total_count += 1;
    }
    pub fn get_mean(&self, offset: u32) -> f64 {
        let mut total: f64 = 0.0;
        let mut count: f64 = 0.0;
        for (idx, c) in self.counts.iter().enumerate() {
            let cnt = *c as f64;
            total += cnt * ((idx as f64) - (offset as f64));
            count += cnt;
        }
        return total / count;
    }
    pub fn get_percentile(&self, offset: u32, percentile: u32) -> u32 {
        let mut total: u64 = 0;
        for c in self.counts.iter() {
            total += *c;
        }
        total = total * (percentile as u64) / 100;
        let mut count = 0;
        for (i, c) in self.counts.iter().enumerate() {
            count += *c;
            if count >= total {
                return (i as u32 - offset) as u32;
            }
        }
        return 0;
    }
}
