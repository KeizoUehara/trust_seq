use std::fs::File;
use std::collections::HashMap;
use trust_seq::utils::FastQReader;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use trust_seq::per_sequence_gc_content::PerSequenceGCContents;
mod trust_seq;

#[test]
fn test_calc_claiming_counts() {
    let mut c = PerSequenceGCContents::new();
    let seq: [u8; 10] = ['a' as u8; 10];
    let qual: [u8; 10] = ['a' as u8; 10];

    let sequence = Sequence {
        sequence: &seq,
        quality: &qual,
    };
    let mut report = File::create("test_data_123.txt").unwrap();
    c.process_sequence(&sequence);
    c.print_text_report(&mut report);
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
