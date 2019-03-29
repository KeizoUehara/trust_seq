pub struct GCModelValue {
    percentage: usize,
    increment: f64,
}
pub struct GCModel {
    models: Vec<Vec<GCModelValue>>,
}
impl GCModel {
    pub fn add_value(&self, gc_count: usize, gc_distribution: &mut [f64]) {
        for val in &self.models[gc_count] {
            gc_distribution[val.percentage] += val.increment;
        }
    }
    pub fn new(read_length: usize) -> GCModel {
        let mut models: Vec<Vec<GCModelValue>> = Vec::new();
        let claiming_counts = calc_claiming_counts(read_length);
        for pos in 0..(read_length + 1) {
            let low_count = (pos as f64 - 0.5).max(0.0);
            let rlen = read_length as f64;
            let high_count = (pos as f64 + 0.5).min(rlen);
            let low_percentage = (low_count * 100.0 / rlen).round() as usize;
            let high_percentage = (high_count * 100.0 / rlen).round() as usize;
            let mut model: Vec<GCModelValue> = Vec::new();
            for p in low_percentage..(high_percentage + 1) {
                model.push(GCModelValue {
                    percentage: p,
                    increment: 1.0 / (claiming_counts[p] as f64),
                });
            }
            models.push(model);
        }
        return GCModel { models: models };
    }
}
pub fn calc_claiming_counts(read_length: usize) -> [u32; 101] {
    let mut claiming_counts = [0; 101];
    for pos in 0..(read_length + 1) {
        let low_count = (pos as f64 - 0.5).max(0.0);
        let rlen = read_length as f64;
        let high_count = (pos as f64 + 0.5).min(rlen);
        let low_percentage = (low_count * 100.0 / rlen).round() as usize;
        let high_percentage = (high_count * 100.0 / rlen).round() as usize;
        for p in low_percentage..(high_percentage + 1) {
            claiming_counts[p] += 1;
        }
    }
    return claiming_counts;
}
