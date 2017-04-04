use std::fs::File;
use trust_seq::utils::FastQReader;
use trust_seq::qc::QCModule;
use trust_seq::gc_model::GCModelValue;
use trust_seq::gc_model::GCModel;
mod trust_seq;

#[test]
fn test_calc_claiming_counts() {
    let c = GCModel::new(10);
    let mut gc_distribution: [f64; 101] = [0.0; 101];
    for gc_content in 0..11 {
        c.add_value(gc_content, &mut gc_distribution);
    }
    for idx in 0..101 {
        assert!(1.0 == gc_distribution[idx]);
    }
}
fn main() {
    let file = File::open("test.fastq").unwrap();
    let mut report = File::create("test_data.txt").unwrap();
    let mut fastq_file = FastQReader::new(file);
    let mut modules = trust_seq::qc::create_qcmodules();
    loop {
        let rslt = fastq_file.next_seq();
        match rslt {
            Ok(Some(seq)) => {
                for module in &mut modules {
                    module.process_sequence(&seq);
                }
            }
            Ok(None) => break,
            Err(e) => {
                println!("Error={}", e);
                break;
            }
        }
    }
    for module in &modules {
        let rslt = module.print_text_report(&mut report);
        match rslt {
            Ok(()) => {}
            Err(e) => {
                println!("Error={}", e);
                break;
            }
        }
    }
}
