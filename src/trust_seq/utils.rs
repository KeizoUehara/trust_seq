use std::io::Read;
use std::io::Result;
use std::option::Option;

pub fn split_by_space(line: &str) -> Vec<&str> {
    let mut vals: Vec<&str> = Vec::new();
    let mut start = -1i32;
    for (idx, ch) in line.char_indices() {
        if start < 0 {
            if !ch.is_whitespace() {
                start = idx as i32;
            }
        } else {
            if ch.is_whitespace() {
                vals.push(&line[(start as usize)..idx]);
                start = -1;
            }
        }
    }
    if 0 < start {
        vals.push(&line[(start as usize)..]);
    }
    return vals;
}

pub fn revcomp(dna: &str) -> String {
    let mut rdna: String = String::with_capacity(dna.len());
    for c in dna.chars().rev() {
        rdna.push(match c {
            'A' => 'T',
            'a' => 't',
            'T' => 'A',
            't' => 'a',
            'G' => 'C',
            'g' => 'c',
            'C' => 'G',
            'c' => 'g',
            _ => 'N',
        });
    }
    rdna
}
pub struct LineReader<R: Read> {
    read: R,
    len: usize,
    pos: usize,
    buff: Vec<u8>,
}
impl<R: Read> LineReader<R> {
    pub fn new(read: R, size: usize) -> LineReader<R> {
        return LineReader {
            read: read,
            len: 0,
            pos: 0,
            buff: vec![0; size],
        };
    }
    fn read(&mut self) -> Result<usize> {
        for i in 0..self.len {
            self.buff[i] = self.buff[self.pos + i];
        }
        self.pos = 0;
        let len = self.read.read(&mut self.buff[self.len..])?;
        self.len = self.len + len;
        return Ok(len);
    }
    pub fn read_line(&mut self) -> Result<Option<&[u8]>> {
        let mut pos = 0;
        for idx in 0.. {
            if self.len <= idx {
                let rslt = self.read()?;
                if 0 == rslt {
                    if idx == 0 {
                        return Ok(None);
                    } else {
                        pos = idx;
                        break;
                    }
                }
            }
            if self.buff[self.pos + idx] == '\n' as u8 {
                pos = idx;
                break;
            }
        }
        let s = &self.buff[self.pos..(self.pos + pos + 1)];
        self.pos = self.pos + pos + 1;
        self.len = self.len - pos - 1;
        return Ok(Some(s));
    }
    pub fn read_lines(&mut self, lsize: usize) -> Result<Vec<&[u8]>> {
        let mut lines = Vec::with_capacity(lsize);
        for idx in 0.. {
            if self.len <= idx {
                let rslt = self.read()?;
                if 0 == rslt {
                    if idx == 0 {
                        return Ok(Vec::new());
                    } else {
                        lines.push(idx);
                        break;
                    }
                }
            }
            if self.buff[self.pos + idx] == '\n' as u8 {
                lines.push(idx);
                if lsize <= lines.len() {
                    break;
                }
            }
        }
        let mut lv = Vec::with_capacity(lsize);
        let mut pos = 0;
        for l in &lines {
            let s = &self.buff[(self.pos + pos)..(self.pos + l + 1)];
            lv.push(s);
            pos = l + 1;
        }
        self.pos = self.pos + pos;
        self.len = self.len - pos;
        return Ok(lv);
    }
}
pub struct Sequence<'a> {
    pub id: &'a [u8],
    pub sequence: &'a [u8],
    pub quality: &'a [u8],
}
pub struct FastQReader<T: Read> {
    reader: LineReader<T>,
}
impl<'a, T: Read> FastQReader<T> {
    pub fn new(read: T) -> FastQReader<T> {
        return FastQReader {
            reader: LineReader::new(read, 4096),
        };
    }
    pub fn next_seq(&mut self) -> Result<Option<Sequence>> {
        let rslt = self.reader.read_lines(4);
        match rslt {
            Ok(lines) => {
                if lines.is_empty() {
                    return Ok(None);
                }
                let id = lines[0];
                let seq = lines[1];
                let qual = lines[3];
                let trim = if lines[0][id.len() - 2] == '\r' as u8 {
                    2
                } else {
                    1
                };
                let len = seq.len() - trim;
                return Ok(Some(Sequence {
                    id: &id[0..id.len() - trim],
                    sequence: &seq[0..len],
                    quality: &qual[0..len],
                }));
            }
            Err(e) => return Err(e),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::split_by_space;
    use super::LineReader;
    #[test]
    fn test_split_by_space() {
        assert_eq!(
            vec!["test1", "test2", "test3"],
            split_by_space("test1 test2  test3")
        );
        assert_eq!(vec!["test1", "test2"], split_by_space(" test1 test2  "));
    }
    #[test]
    fn test_split_by_newline() {
        let cur = std::io::Cursor::new(b"test1\ntest2\ntest3\n");
        let mut reader = LineReader::new(cur, 1024);
        println!(
            "{}",
            String::from_utf8_lossy(reader.read_line().unwrap().unwrap())
        );
        println!(
            "{}",
            String::from_utf8_lossy(reader.read_line().unwrap().unwrap())
        );
        println!(
            "{}",
            String::from_utf8_lossy(reader.read_line().unwrap().unwrap())
        );
        assert_eq!(1, 1);
    }
}
