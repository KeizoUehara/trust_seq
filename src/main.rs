#[macro_use]
extern crate serde_derive;

extern crate serde_json;

extern crate getopts;

use std::fs::File;
use std::env;
use serde_json::map::Map;
use trust_seq::qc;
use trust_seq::qc::QCReport;
use trust_seq::utils::FastQReader;
use trust_seq::trust_seq::TrustSeqConfig;
mod trust_seq;

fn main() {

    let mut report = File::create("test_data.txt").unwrap();
    let rslt = TrustSeqConfig::get_fastqc_config(&env::args().collect());
    let config: TrustSeqConfig;
    match rslt {
        Ok(c) => {
            config = c;
        }
        Err(e) => {
            println!("Error:{:?}", e);
            return;
        }
    }
    let file = File::open(&config.files[0]).unwrap();
    {
        let mut fastq_file = FastQReader::new(file);
        let mut modules = qc::create_qcmodules(&config);
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
        match qc::write_text_reports(&modules, &mut report) {
            Err(e) => {
                println!("Error={:?}", e);
            }
            _ => {}
        };
        //        let json_report = qc::get_json_reports(&modules);
        //       match json_report {
        //          Ok(map) => {
        //             let json = serde_json::to_string_pretty(&map).unwrap();
        //             println!("{}", json);
        //        }
        //       Err(e) => {
        //           println!("Error={:?}", e);
        //       }
        // }
    }
}
