use trust_seq::utils;
use std::io::BufRead;
use std::vec::Vec;
use std::cmp;

#[derive(Debug)]
pub struct Adapter {
    pub name: String,
    sequence: String,
}
impl Adapter {
    pub fn new(name: &str, sequence: &str) -> Adapter {
        return Contaminant {
                   name: name.to_string(),
                   sequence: sequence.to_string(),
               };
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
            cons.push(Adaper {
                          name: key,
                          sequence: value.clone(),
                      });
        }
        return cons;
    }
}
