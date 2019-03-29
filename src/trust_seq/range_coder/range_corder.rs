#[derive(Clone, Copy, Debug)]
struct SymFreqs {
    symbol: u16,
    freq: u16,
}
const MAX_FREQ: u32 = ((1u32 << 16) - 32) as u32;
const TOP: u32 = (1 << 24);
#[derive(Debug)]
struct RangeEncoder {
    low: u64,
    range: u32,
    code: u32,
    buf: Vec<u8>,
}
impl RangeEncoder {
    fn new(buff_size: usize) -> RangeEncoder {
        return RangeEncoder {
            low: 0,
            range: std::u32::MAX,
            code: 0,
            buf: Vec::new(),
        };
    }
    fn encode(&mut self, cum_freq: u32, freq: u32, total_freq: u32) {
        self.range /= total_freq;
        self.low += cum_freq as u64 * self.range as u64;
        self.range *= freq;
        assert!(cum_freq + freq <= total_freq);
        while self.range < TOP {
            let m = ((self.low ^ (self.low + self.range as u64)) >> 54) as u32;
            if m != 0 {
                self.range = self.low as u32 | (TOP - 1) - (self.low as u32)
            }
            self.buf.push((self.low >> 56) as u8);
            self.range <<= 8;
            self.low <<= 8;
        }
    }
    fn finish_encode(&mut self) -> () {
        for _ in 0..8 {
            self.buf.push((self.low >> 56) as u8);
            self.low <<= 8;
        }
    }
}
#[derive(Debug)]
struct RangeDecoder<'a> {
    low: u64,
    range: u32,
    code: u32,
    idx: usize,
    buf: &'a [u8],
}
impl<'a> RangeDecoder<'a> {
    fn new(buf: &'a [u8]) -> RangeDecoder<'a> {
        let mut code: u32 = 0;
        let mut idx: usize = 0;
        for _ in 0..8 {
            code = code << 8 | buf[idx] as u32;
            idx += 1;
        }
        return RangeDecoder {
            low: 0,
            range: std::u32::MAX,
            code: code,
            idx: idx,
            buf: buf,
        };
    }
    fn decode(&mut self, cum_freq: u32, freq: u32, total_freq: u32) {
        let temp: u32 = cum_freq * self.range;
        self.low += temp as u64;
        self.code -= temp;
        self.range *= freq;
        while self.range < TOP {
            let tmp = self.low ^ (self.low + self.range as u64);
            if (tmp >> 56) != 0 {
                let old_range = self.range;
                self.range = (self.low as u32 | (TOP - 1)) - (self.low as u32);
            }
            self.code = (self.code << 8) | (self.buf[self.idx] as u32);
            self.idx += 1;
            self.range <<= 8;
            self.low <<= 8;
        }
    }
    fn get_freq(&mut self, total_freq: u32) -> u32 {
        self.range /= total_freq;
        return self.code / self.range;
    }
}
#[derive(Debug)]
struct SimpleModel {
    total_freq: u32,
    bub_count: u32,
    freqs: Vec<SymFreqs>,
}

impl SimpleModel {
    fn new(num_of_symbol: usize) -> SimpleModel {
        let mut c = SimpleModel {
            total_freq: num_of_symbol as u32,
            bub_count: 0,
            freqs: vec![SymFreqs { symbol: 0, freq: 0 }; num_of_symbol + 1],
        };
        for (idx, freq) in &mut c.freqs.iter_mut().enumerate() {
            if idx == 0 {
                freq.symbol = 0;
                freq.freq = MAX_FREQ as u16;
            } else {
                freq.symbol = (idx - 1) as u16;
                freq.freq = 1;
            }
        }
        return c;
    }
    fn normalize(&mut self) -> () {
        self.total_freq = 0;
        for freq in &mut self.freqs[1..] {
            freq.freq -= freq.freq >> 1;
            self.total_freq += freq.freq as u32;
        }
    }
    fn encode(&mut self, rc: &mut RangeEncoder, sym: u16) {
        let mut acc_freq: u32 = 0;
        let mut idx: usize = 1;
        while self.freqs[idx].symbol != sym {
            acc_freq += self.freqs[idx].freq as u32;
            idx += 1;
        }
        rc.encode(acc_freq, self.freqs[idx].freq as u32, self.total_freq);
        self.freqs[idx].freq += 8;
        self.total_freq += 8;
        if self.total_freq > MAX_FREQ {
            self.normalize();
        }
        self.bub_count += 1;
        if self.bub_count & 15 == 0 && self.freqs[idx].freq > self.freqs[idx - 1].freq {
            let t = self.freqs[idx];
            self.freqs[idx] = self.freqs[idx - 1];
            self.freqs[idx - 1] = t;
        }
    }
    fn decode(&mut self, rd: &mut RangeDecoder) -> u16 {
        let mut acc_freq: u32 = 0;
        let mut idx: usize = 1;
        let freq = rd.get_freq(self.total_freq);
        loop {
            acc_freq += self.freqs[idx].freq as u32;
            if acc_freq > freq {
                break;
            }
            idx += 1;
        }
        acc_freq -= self.freqs[idx].freq as u32;
        rd.decode(acc_freq, self.freqs[idx].freq as u32, self.total_freq);
        self.freqs[idx].freq += 8;
        self.total_freq += 8;
        if self.total_freq > MAX_FREQ {
            self.normalize();
        }
        self.bub_count += 1;
        if self.bub_count & 15 == 0 && self.freqs[idx].freq > self.freqs[idx - 1].freq {
            let t = self.freqs[idx];
            self.freqs[idx] = self.freqs[idx - 1];
            self.freqs[idx - 1] = t;
            return t.symbol;
        }
        return self.freqs[idx].symbol;
    }
}
#[cfg(test)]
mod tests {
    use super::RangeDecoder;
    use super::RangeEncoder;
    use super::SimpleModel;
    #[test]
    fn test_encode_decode() {
        let test_text = "this is encoder test";
        let mut rc = RangeEncoder::new(4096);
        let mut simple_model = SimpleModel::new(128);
        for ch in test_text.chars() {
            simple_model.encode(&mut rc, ch as u16);
        }
        rc.finish_encode();
        let mut simple_model2 = SimpleModel::new(128);
        let mut rd = RangeDecoder::new(rc.buf.as_slice());
        println!("");
        let mut buf: Vec<u8> = Vec::new();
        for _ in 0..test_text.len() {
            buf.push(simple_model2.decode(&mut rd) as u8);
        }
        assert_eq!(test_text, String::from_utf8(buf).unwrap());
    }
}
