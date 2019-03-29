use std::io::BufRead;
use std::vec::Vec;

#[derive(Debug)]
pub struct Adapter {
    pub name: String,
    pub sequence: String,
    pub positions: Vec<u64>,
}
impl Adapter {
    pub fn new(name: &str, sequence: &str) -> Adapter {
        return Adapter {
            name: name.to_string(),
            sequence: sequence.to_string(),
            positions: Vec::new(),
        };
    }
    pub fn increment_count(&mut self, idx: usize) {
        for pos in &mut self.positions[idx..] {
            *pos += 1;
        }
    }
    pub fn load_adapters<R: BufRead>(reader: R) -> Vec<Adapter> {
        let mut cons = Vec::new();
        for rslt in reader.lines() {
            let line = match rslt {
                Ok(l) => l,
                _ => String::new(),
            };
            if line.starts_with('#') || line.len() == 0 {
                continue;
            }
            let mut key = String::new();
            let mut value = String::new();
            match line.find('\t') {
                Some(i) => key.push_str(&line[0..i]),
                None => continue,
            }
            match line.rfind('\t') {
                Some(i) => value.push_str(&line[(i + 1)..]),
                None => continue,
            }
            cons.push(Adapter::new(&key, &value));
        }
        return cons;
    }
}
