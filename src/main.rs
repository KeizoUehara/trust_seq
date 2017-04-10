extern crate getopts;
use std::fs::File;
use trust_seq::utils::FastQReader;
mod trust_seq;

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
