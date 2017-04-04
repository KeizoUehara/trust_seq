use std::io::Write;
use std::io::Result;
use std::collections::hash_map::HashMap;
use trust_seq::gc_model::GCModel;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;

pub struct PerSequenceGCContents {
    gc_distribution: [f64; 101],
    gc_models: HashMap<usize, Box<GCModel>>,
}
impl PerSequenceGCContents {
    pub fn new() -> PerSequenceGCContents {
        return PerSequenceGCContents {
                   gc_distribution: [0; 101],
                   gc_models: HashMap::new(),
               };
    }
}

impl QCModule for PerSequenceGCContents {
    fn print_report(&mut self) -> () {}
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        for idx in 0..101 {
            println!("{}={}", idx, self.gc_distribution);
        }
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        let mut gc_count: usize = 0;
        for s in seq.sequence {
            let ch = *s as char;
            let is_gc = match ch {
                'G' => true,
                'g' => true,
                'c' => true,
                'C' => true,
            };
            if is_gc {
                gc_count += 1;
            }
        }

        if !self.gc_models.contains_key(seq.len()) {
            self.gc_models[seq.len()] = Box::new(GCModel::new(seq.len()));
        }
        self.gc_models[seq.len()].addValue(gc_count, &mut self.gc_distribution);
    }
}
