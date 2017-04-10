use std::cmp;
use std::io::Write;
use std::io::Result;
use trust_seq::utils::Sequence;
use trust_seq::qc::QCModule;
use trust_seq::qc::PhreadEncoding;

#[derive(Debug)]
pub struct AdapterContent {
    longest_sequence: u32,
    longest_adapter: u32,
    total_count: u64,
    lowest_char: u8,
    gatcn_count: [u64; 5],
}
impl BasicStats {
    pub fn new() -> BasicStats {
        return BasicStats {
                   actual_count: 0,
                   filtered_count: 0,
                   min_length: 0,
                   max_length: 0,
                   lowest_char: 255,
                   gatcn_count: [0; 5],
               };
    }
}

impl QCModule for BasicStats {
    fn print_report(&mut self) -> () {
        println!("BasicStats={:?}", self);
        println!("Encode = {:?}",
                 PhreadEncoding::get_phread_encoding(self.lowest_char));
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<()> {
        let encoding = PhreadEncoding::get_phread_encoding(self.lowest_char).unwrap();
        try!(write!(writer, "Encoding\t{}\n", encoding.name));
        try!(write!(writer, "Total Sequences\t{}\n", self.actual_count));
        try!(write!(writer, "Filtered Sequences\t{}\n", self.filtered_count));
        if self.min_length == self.max_length {
            try!(write!(writer, "Sequence length\t{}\n", self.min_length));
        } else {
            try!(write!(writer,
                        "Sequence length\t{}-{}\n",
                        self.min_length,
                        self.max_length));
        }
        let gc_count = self.gatcn_count[0] + self.gatcn_count[3];
        let at_count = self.gatcn_count[1] + self.gatcn_count[2];
        try!(write!(writer,
                    "%GC\t{}\n",
                    (gc_count * 100) / (gc_count + at_count)));
        return Ok(());
    }
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        self.actual_count += 1;
        let len = seq.sequence.len() as u32;
        if self.actual_count == 1 {
            self.min_length = len;
            self.max_length = len;
        } else {
            self.min_length = cmp::min(self.min_length, len);
            self.max_length = cmp::max(self.min_length, len);
        }
        for q in seq.sequence {
            let ch = *q as char;
            let idx = match ch {
                'G' => 0,
                'A' => 1,
                'T' => 2,
                'C' => 3,
                'g' => 0,
                'a' => 1,
                't' => 2,
                'c' => 3,
                'N' => 4,
                _ => {
                    println!("unexpected char={}", ch);
                    4
                }
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
