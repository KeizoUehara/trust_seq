use std::cmp;
use std::io::BufRead;
use std::vec::Vec;
use trust_seq::utils;

#[derive(Debug, Clone, Copy)]
pub struct ContaminantHit<'a> {
    pub contaminant: &'a Contaminant,
    pub direction: u32,
    pub length: usize,
    pub percent_id: u32,
}
fn get_length<'a>(h: Option<ContaminantHit<'a>>) -> usize {
    if let Some(x) = h {
        x.length
    } else {
        0
    }
}
pub fn find_contaminant<'a>(
    contaminants: &'a Vec<Contaminant>,
    query: &[u8],
) -> Option<ContaminantHit<'a>> {
    match contaminants
        .iter()
        .map(|c| c.find_match(query))
        .max_by_key(|h| get_length(*h))
    {
        Some(v) => v,
        None => None,
    }
}
#[derive(Debug)]
pub struct Contaminant {
    pub name: String,
    forward: String,
    reverse: String,
}
#[derive(Debug, PartialEq)]
struct Hit {
    start: usize,
    length: usize,
    mismatch: usize,
}
fn find_longest_match_with_one_mismatch(a: &[u8], b: &[u8]) -> Option<Hit> {
    assert!(a.len() == b.len());
    let mut start: usize = 0;
    let mut next_start: Option<usize> = None;
    let mut match_region: Option<(usize, usize, usize)> = None;
    for idx in 0..a.len() {
        if a[idx] != b[idx] {
            let is_next_match = idx + 1 < a.len() && a[idx + 1] == b[idx + 1];
            if start < idx && is_next_match {
                if let Some(x) = next_start {
                    start = x;
                }
                next_start = Some(idx + 1);
            } else {
                start = idx + 1;
                next_start = None;
            }
        } else {
            let prev_len = if let Some(m) = match_region {
                m.1 - m.0
            } else {
                0
            };
            if prev_len < idx - start {
                match_region = Some((start, idx, if next_start == None { 0 } else { 1 }));
            }
        }
    }
    return match match_region {
        Some(m) => Some(Hit {
            start: m.0,
            length: m.1 - m.0 + 1,
            mismatch: m.2,
        }),
        None => None,
    };
}
#[cfg(test)]
mod tests {
    use super::find_contaminant;
    use super::find_longest_match_with_one_mismatch;
    use super::Contaminant;
    fn test_match(lhs: &str, rhs: &str) -> Option<(usize, usize, usize)> {
        let hit = find_longest_match_with_one_mismatch(lhs.as_bytes(), rhs.as_bytes());
        return match hit {
            Some(h) => Some((h.start, h.length, h.mismatch)),
            None => None,
        };
    }

    #[test]
    fn test_find_longest_match_with_one_mismatch() {
        assert_eq!(None, test_match("AAAAAA", "BBBBBB"));
        assert_eq!(Some((4, 2, 0)), test_match("AAAAAA", "BBBBAA"));
        assert_eq!(Some((0, 3, 0)), test_match("AAAAAA", "AAABBB"));
        assert_eq!(Some((5, 3, 0)), test_match("AAAAAAAAA", "BBBBBAAAB"));
        assert_eq!(Some((5, 3, 0)), test_match("AAAAAAAAA", "BAABBAAAB"));
        assert_eq!(
            Some((5, 8, 1)),
            test_match("OOOOOOOOOOOOOO", "OOOOXOOXOOOOOX")
        );
        assert_eq!(
            Some((0, 13, 1)),
            test_match("OOOOOOOOOOOOOO", "OOOOXOOOOOOOOX")
        );
    }
    #[test]
    fn test_find_match() {
        let c = Contaminant::new("Test", "AGCTTCGA");
        let hit = c.find_match("AGCTTCGA".as_bytes());
        assert_eq!(0, hit.unwrap().direction);
        let hit2 = c.find_match("TCGAAGCT".as_bytes());
        assert_eq!(1, hit2.unwrap().direction);
    }
    #[test]
    fn test_find_match3() {
        let c = Contaminant::new(
            "Illumina Paried End PCR Primer 1",
            "AATGATACGGCGACCACCGAGATCTACACTCTTTCCCTACACGACGCTCTTCCGATCT",
        );
        let hit = c.find_match("ACACTCTTTCCCTACACGACGCTCTTCCGATCT".as_bytes());
        println!("hit = {:?}", hit);
    }
    #[test]
    fn test_find_match2() {
        let c = Contaminant::new(
            "Illumina Single End Adapter 1",
            "GATCGGAAGAGCTCGTATGCCGTCTTCTGCTTG",
        );
        let hit = c.find_match("GATAGATGATCGGAAGAGCTCGTATGCCGTCTTCTGCTTGGATAGA".as_bytes());
        assert_eq!(33, hit.unwrap().length);
        let hit2 = c.find_match("AAACAAGCAGAAGACGGCATACGAGCTCTTCCGATCAAA".as_bytes());
        assert_eq!(33, hit2.unwrap().length);
    }
}
impl Contaminant {
    pub fn new(name: &str, sequence: &str) -> Contaminant {
        return Contaminant {
            name: name.to_string(),
            forward: sequence.to_string(),
            reverse: utils::revcomp(sequence),
        };
    }
    pub fn find_match(&self, query: &[u8]) -> Option<ContaminantHit> {
        if 8 <= query.len() && query.len() < 20 {
            if self.forward.as_bytes() == query {
                return Some(ContaminantHit {
                    contaminant: self,
                    direction: 0,
                    length: query.len(),
                    percent_id: 100,
                });
            } else if self.reverse.as_bytes() == query {
                return Some(ContaminantHit {
                    contaminant: self,
                    direction: 1,
                    length: query.len(),
                    percent_id: 100,
                });
            }
        }
        let s: i32 = 20 - self.forward.len() as i32;
        let e: i32 = (query.len() - 20) as i32;
        let mut best_len = 20;
        let mut best_hit: Option<ContaminantHit> = None;
        for (idx, seq) in [&self.forward, &self.reverse].iter().enumerate() {
            for offset in s..e {
                let start1 = cmp::max(0, -offset) as usize;
                let end1 = cmp::min(seq.len() as i32, query.len() as i32 - offset) as usize;
                let start2 = cmp::max(0, offset) as usize;
                let end2 = cmp::min(query.len() as i32, seq.len() as i32 + offset) as usize;
                let hit = find_longest_match_with_one_mismatch(
                    &seq.as_bytes()[start1..end1],
                    &query[start2..end2],
                );
                match hit {
                    Some(h) => {
                        if best_len < h.length {
                            let pid = (h.length - h.mismatch) * 100 / h.length;
                            best_hit = Some(ContaminantHit {
                                contaminant: self,
                                direction: idx as u32,
                                length: h.length,
                                percent_id: pid as u32,
                            });
                            best_len = h.length;
                        }
                    }
                    _ => {}
                };
            }
        }
        return best_hit;
    }
    pub fn load_contaminants<R: BufRead>(reader: R) -> Vec<Contaminant> {
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
            cons.push(Contaminant {
                name: key,
                forward: value.clone(),
                reverse: utils::revcomp(&value),
            });
        }
        return cons;
    }
}
