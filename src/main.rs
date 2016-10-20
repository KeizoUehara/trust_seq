use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Error;
use std::fs::File;
use std::io::ErrorKind;
use std::boxed::Box;
use fastqc::QCModule;
use fastqc::Sequence;

mod fastqc;

struct FastQFile<T:Read>{
    reader: Box<BufReader<T>>,
    sequence: String,
    quality: String,
    name: String,
}

impl <T:Read> FastQFile<T> {
    fn new(read:T) -> FastQFile<T>{
        let fastq = FastQFile {
            reader: Box::new(BufReader::new(read)),
            sequence: "".to_string(),
            quality: "".to_string(),
            name: "".to_string(),
        };
        return fastq;
    }
    fn next(&mut self) -> Result<Sequence,Error> {
        
        let n = try!(self.reader.read_to_string(&mut self.name));
        if n <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n2 = try!(self.reader.read_to_string(&mut self.sequence));
        if n2 <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n3 = try!(self.reader.read_to_string(&mut self.name));
        if n3 <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n4 = try!(self.reader.read_to_string(&mut self.quality));
        if n4 <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }else{
            return Ok(Sequence {
                sequence : &self.sequence.trim().as_bytes(),
                quality : &self.quality.trim().as_bytes(),
            });
        }
        
    }
}
fn main() {
    let file = File::open("test.fastq").unwrap();
    let mut fastq_file = FastQFile::new(file);
    let mut report1 = fastqc::BasicStats::new();
    let mut report2 = fastqc::PerBaseQualityScores::new();
    loop {
        let rslt = fastq_file.next();
        match rslt {
            Ok(seq) => {
                println!("seq={}!",String::from_utf8_lossy(seq.sequence));
                report1.process_sequence(&seq);
                report2.process_sequence(&seq);
            },
            Err(e) => {
                println!("Error={}",e);
                break;
            }
        }
    }
    report1.print_report();
    report2.print_report();
}
