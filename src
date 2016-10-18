use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Error;
use std::fs::File;
use std::io::ErrorKind;
use std::boxed::Box;

#[derive(Copy)]
struct QualityCount {
    counts: [u64;150],
}
impl QualityCount {
    fn new() -> QualityCount {
        return QualityCount { counts: [0;150]}; 
    }
}
impl Clone for QualityCount{
    fn clone(&self) -> QualityCount{
        return *self;
    }
}
struct PerBaseQualityScores{
    quality_counts: Vec<QualityCount>
}
impl PerBaseQualityScores{
    fn new() -> PerBaseQualityScores {
        return PerBaseQualityScores {quality_counts: Vec::new()};
    }
}
#[derive(Debug)]
struct BasicStats {
    actual_count:u64,
    filtered_count:u64,
    min_length:u32,
    max_length:u32,
    lowest_char:u8,
    gatcn_count:[u8;5]
}
trait QCModule{
    fn process_sequence(&mut self,seq:&Sequence) -> ();
    fn print_report(&mut self) -> ();
}
impl QCModule for PerBaseQualityScores {
    fn print_report(&mut self) -> () {
        for ch in &self.quality_counts {
            for q in ch.counts.iter() {
                print!("{} ",q);
            }
            println!("");
        }
    }
    fn process_sequence(&mut self,seq:&Sequence) -> (){
        let len = seq.quality.len();
        if self.quality_counts.len() < len {
            self.quality_counts.resize(len,QualityCount::new());
        }
        for (idx,ch) in seq.quality.iter().enumerate() {
            self.quality_counts[idx].counts[*ch as usize] += 1;
        }
    }
}
impl BasicStats {
    pub fn new() -> BasicStats{
        return BasicStats {
            actual_count : 0,
            filtered_count: 0,
            min_length: 0,
            max_length: 0,
            lowest_char: 255,
            gatcn_count: [0;5]
        }
    }
}
impl QCModule for BasicStats{
    fn print_report(&mut self) -> () {
        println!("BasicStats={:?}",self);
    }
    fn process_sequence(&mut self,seq:&Sequence) -> () {
        self.actual_count += 1;
        let len = seq.sequence.len() as u32;
        if self.actual_count == 1 {
            self.min_length = len;
            self.max_length = len;
        }else{
            self.min_length = std::cmp::min(self.min_length,len);
            self.max_length = std::cmp::max(self.min_length,len);
        }
        for q in seq.sequence {
            let ch = *q as char;
            let idx = match ch {
                'G'=> 0,
                'A'=> 1,
                'T'=> 2,
                'C'=> 3,
                'g'=> 0,
                'a'=> 1,
                't'=> 2,
                'c'=> 3,
                'N'=> 4,
                _ => {println!("unexpected char={}",ch);4},
            };
            self.gatcn_count[idx] += 1;
        }
        for q in seq.quality {
            if *q < self.lowest_char {
                self.lowest_char = *q;
            }
        }
    }
}
struct Sequence<'a> {
    sequence: &'a [u8],
    quality: &'a [u8],
   
}
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
        
        let n = try!(self.reader.read_line(&mut self.name));
        if n <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n2 = try!(self.reader.read_line(&mut self.sequence));
        if n2 <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n3 = try!(self.reader.read_line(&mut self.name));
        if n3 <= 0 {
            return Err(Error::new(ErrorKind::Other,"No"));
        }
        let n4 = try!(self.reader.read_line(&mut self.quality));
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
    let mut report1 = BasicStats::new();
    let mut report2 = PerBaseQualityScores::new();
    loop {
        let rslt = fastq_file.next();
        match rslt {
            Ok(seq) => {
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
