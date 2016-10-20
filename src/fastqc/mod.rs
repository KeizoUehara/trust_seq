use std::cmp;
pub struct Sequence<'a> {
    pub sequence: &'a [u8],
    pub quality: &'a [u8],
}

pub trait QCModule{    
    fn process_sequence(&mut self,seq:&Sequence) -> ();
    fn print_report(&mut self) -> ();
}
#[derive(Copy)]
struct QualityCount {
    counts: [u64;150],
}
impl QualityCount {
    fn new() -> QualityCount {
        return QualityCount { counts: [0;150]}; 
    }
    pub fn get_mean(&self,offset :u32) -> f64 {
        let mut total: f64 = 0.0;
        let mut count: f64 = 0.0;
        for (idx,c) in self.counts.iter().enumerate() {
            let cnt = *c as f64;
            total += cnt * ((idx as f64) - (offset as f64));
            count += cnt;
        }
        return total / count;
    }
    pub fn get_percntile(&self,offset :u32, percentile:u32) -> u32{
        let mut total :u64 = 0;
        for c in self.counts.iter(){
            total += *c;
        }
        total = total * (percentile as u64)/ 100;
        let mut count = 0;
        for (i,c) in self.counts.iter().enumerate() {
            count += *c;
            if count >= total {
                return (i as u32 - offset) as u32;
            }
        }
        return 0;
    }
}
impl Clone for QualityCount{
    fn clone(&self) -> QualityCount{
        return *self;
    }
}
pub struct PerBaseQualityScores{
    min_char :u8,
    max_char :u8,
    quality_counts: Vec<QualityCount>
}
impl PerBaseQualityScores{
    pub fn new() -> PerBaseQualityScores {
        return PerBaseQualityScores {
            quality_counts: Vec::new(),
            min_char: 255,
            max_char: 0
        };
    }
}
#[derive(Debug)]
pub struct BasicStats {
    actual_count:u64,
    filtered_count:u64,
    min_length:u32,
    max_length:u32,
    lowest_char:u8,
    gatcn_count:[u8;5]
}
#[derive(Clone,Debug)]
pub struct PhreadEncoding {
    name: &'static str,
    offset: u8
}
static SANGER_ENCODING_OFFSET:u8 = 33;
static ILUMINA_1_3_ENCODING_OFFSET:u8 = 64;

impl PhreadEncoding {
    fn get_phread_encoding(lowest_char:u8) -> Result<PhreadEncoding,String> {
        if lowest_char < 32 {
            return Err("No known encodings with chars < 33".to_string());
        }else if lowest_char < 64{
            return Ok(PhreadEncoding{
                offset: SANGER_ENCODING_OFFSET,
                name:  "Sanger / Illumina 1.9"
            });
        }else if lowest_char == ILUMINA_1_3_ENCODING_OFFSET + 1 {
            return Ok(PhreadEncoding{
                offset: ILUMINA_1_3_ENCODING_OFFSET ,
                name:  "Illumina 1.3"
            });
        }else if lowest_char <= 126 {
            return Ok(PhreadEncoding{
                offset: ILUMINA_1_3_ENCODING_OFFSET ,
                name:  "Illumina 1.5"
            });
        }else{
            return Err("No known encodings with chars > 126".to_string());
        }
    }
    
}
impl QCModule for PerBaseQualityScores {
    fn print_report(&mut self) -> () {
        for ch in &self.quality_counts {
            println!("mean = {}",ch.get_mean(self.min_char as u32));
        }
    }
    fn process_sequence(&mut self,seq:&Sequence) -> (){
        let len = seq.quality.len();
        if self.quality_counts.len() < len {
            self.quality_counts.resize(len,QualityCount::new());
        }
        for (idx,ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char,*ch);
            self.max_char = cmp::max(self.max_char,*ch);            
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
        println!("Encode = {:?}",PhreadEncoding::get_phread_encoding(self.lowest_char));
    }
    fn process_sequence(&mut self,seq:&Sequence) -> () {
        self.actual_count += 1;
        let len = seq.sequence.len() as u32;
        if self.actual_count == 1 {
            self.min_length = len;
            self.max_length = len;
        }else{
            self.min_length = cmp::min(self.min_length,len);
            self.max_length = cmp::max(self.min_length,len);
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
