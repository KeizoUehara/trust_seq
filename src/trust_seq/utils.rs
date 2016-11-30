use std::io::Read;
use std::boxed::Box;
use std::io::Result;
use std::fs::File;
use std::option::Option;
use std::io::Error;
use std::io::ErrorKind;

pub struct LineReader<R:Read> {
    read: Box<R>,
    len: usize,
    pos: usize,
    buff: Vec<u8>,
}
impl <R:Read> LineReader<R> {
    pub fn new(read: R,size:usize) -> LineReader<R> {
        return LineReader {
            read: Box::new(read),
            len: 0,
            pos: 0,
            buff: vec![ 0 ;size],
        }
    }
    pub fn read_lines<'b>(&'b mut self,lines:&mut [&'b [u8]]) -> Result<bool> {
        let line_num = lines.len();
        loop {
            let mut positions = Vec::new();
            let mut start = self.pos;
            for line_idx in 0..line_num{
                for idx in start .. self.len {
                    if self.buff[idx] == '\n' as u8 {
                        positions.push(start .. (idx + 1));
                        start = idx + 1;
                        break;
                    }
                }
            }
            if positions.len() == lines.len() {
                for (idx,line) in lines.iter_mut().enumerate(){
                    *line = &self.buff[positions[idx].start ..
                                       positions[idx].end];
                }
                self.pos = start;
                return Ok(true);
            }
            let l = self.len - self.pos;
            for idx in 0 .. l{
                self.buff[l - idx - 1] = self.buff[self.len - idx - 1];
            }
            self.pos = 0;
            self.len = l;
            let len = try!(self.read.read(&mut self.buff[self.len ..]));
            if len <= 0{
                if self.len == 0 {
                    return Ok(false);
                }else{
                    self.len = 0;
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,"Unexpectedeof"));
                }
            }
            self.len = self.len + len;
        }
    }
    pub fn read_line(&mut self) -> Result<Option<&[u8]>> {
        loop {
            let start = self.pos;
            for idx in self.pos .. self.len {
                if self.buff[idx] == '\n' as u8 {
                    self.pos = idx + 1;
                    return Ok(Some(& self.buff[start .. self.pos]));
                }
            }
            let l = self.len - self.pos;
            for idx in 0 .. l{
                self.buff[l - idx - 1] = self.buff[self.len - idx - 1];
            }
            self.pos = 0;
            self.len = l;
            let len = try!(self.read.read(&mut self.buff[self.len ..]));
            if len <= 0{
                if self.len == 0 {
                    return Ok(None);
                }else{
                    let l = self.len;
                    self.len = 0;
                    return Ok(Some(& self.buff[0 .. l]));
                }
            }
            self.len = self.len + len;
        }
    }
}
pub struct Sequence<'a>{
    pub sequence: &'a [u8],
    pub quality: &'a [u8],
}
pub struct FastQReader<'a,T:Read>{
    reader: LineReader<T>,
    lines: [&'a [u8];4],
}
static U8_ARRAY:[u8;1] = ['a' as u8;1];
impl <'a,T:Read> FastQReader<'a,T>{
    pub fn new(read:T) -> FastQReader<'a,T>{
        return FastQReader {
            reader : LineReader::new(read,4096),
            lines: [&U8_ARRAY[0..0];4],
        };
    }
    pub fn next_seq(&mut self) -> Result<Option<Sequence>> {
        let rslt = try!(self.reader.read_lines(&mut self.lines));
        match rslt {
            true => {
                let seq = self.lines[1];
                let qual = self.lines[3];
                let len = seq.len() - 1;
                return Ok(Some(Sequence{
                    sequence: &seq[0..len],
                    quality: &qual[0..len]
                }));
            },
            false => return Ok(None),
        }
    }
}
