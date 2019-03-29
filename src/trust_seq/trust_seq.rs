use super::group::GroupType;
use super::module_config::ModuleConfig;
use getopts::{Fail, Options};
use serde_json;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::Path;
use std::vec::Vec;
#[derive(Debug)]
pub enum TrustSeqErr {
    Io(io::Error),
    ArgError(Fail),
    JsonError(serde_json::error::Error),
}
impl From<io::Error> for TrustSeqErr {
    fn from(err: io::Error) -> TrustSeqErr {
        TrustSeqErr::Io(err)
    }
}
impl From<serde_json::error::Error> for TrustSeqErr {
    fn from(err: serde_json::error::Error) -> TrustSeqErr {
        TrustSeqErr::JsonError(err)
    }
}
impl From<Fail> for TrustSeqErr {
    fn from(err: Fail) -> TrustSeqErr {
        TrustSeqErr::ArgError(err)
    }
}

#[derive(Clone, Debug)]
pub struct TrustSeqConfig {
    pub group_type: GroupType,
    pub quiet: bool,
    pub show_version: bool,
    pub kmer_size: Option<u32>,
    pub output_dir: String,
    pub casava: bool,
    pub nano: bool,
    pub nofilter: bool,
    pub contaminant_file: Option<String>,
    pub adapter_file: Option<String>,
    pub module_config: ModuleConfig,
    pub files: Vec<String>,
}
#[test]
fn test_get_fastqc_config() {
    let m = vec![
        "trust_seq".to_string(),
        "-c".to_string(),
        "unknown_file.fastq".to_string(),
    ];
    let result = TrustSeqConfig::get_fastqc_config(&m);
    println!("{:?}", result);
}
impl TrustSeqConfig {
    pub fn get_fastqc_config(args: &Vec<String>) -> Result<TrustSeqConfig, TrustSeqErr> {
        let mut opts = Options::new();
        opts.optflag("h", "help", "print this help menu");
        opts.optopt(
            "c",
            "contaminant_file",
            "Contaminant file path",
            "CONTAMINAIT_FILE",
        );
        opts.optopt("a", "adapter_file", "adapter file path", "ADAPTER_FILE");
        opts.optopt("l", "limit_file", "limit file path", "LIMIT_FILE");
        let mut config: TrustSeqConfig = TrustSeqConfig::new();
        let matches = opts.parse(&args[1..])?;
        if let Some(c_path) = matches.opt_str("c") {
            if !Path::new(&c_path).is_file() {
                return Err(TrustSeqErr::Io(io::Error::new(
                    ErrorKind::NotFound,
                    format!("{} is not Found!", c_path),
                )));
            }
            config.contaminant_file = Some(c_path);
        }
        if let Some(a_path) = matches.opt_str("a") {
            if !Path::new(&a_path).is_file() {
                return Err(TrustSeqErr::Io(io::Error::new(
                    ErrorKind::NotFound,
                    format!("{} is not Found!", a_path),
                )));
            }
            config.adapter_file = Some(a_path);
        }
        if let Some(l_path) = matches.opt_str("l") {
            let f = File::open(l_path)?;
            config.module_config.load(BufReader::new(f))?;
        }
        if matches.free.len() <= 0 {
            return Err(TrustSeqErr::Io(io::Error::new(
                ErrorKind::NotFound,
                format!("missing orepand"),
            )));
        }
        config.files = matches.free;
        return Ok(config);
    }
    pub fn new() -> TrustSeqConfig {
        return TrustSeqConfig {
            group_type: GroupType::Linear,
            quiet: false,
            show_version: false,
            kmer_size: None,
            output_dir: ".".to_string(),
            casava: false,
            nano: false,
            nofilter: false,
            contaminant_file: None,
            adapter_file: None,
            module_config: ModuleConfig::new(),
            files: Vec::new(),
        };
    }
}
