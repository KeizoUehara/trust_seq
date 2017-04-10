extern crate getopts;
mod trust_seq;

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use trust_seq::contaminant::Contaminant;
    use trust_seq::contaminant_list::CONTAMINANT_LIST;
    #[test]
    fn test_load_contaminants() {
        let cons = Contaminant::load_contaminants(BufReader::new(CONTAMINANT_LIST.as_bytes()));
        assert_eq!(151, cons.len());
        print!("{:?}", cons[0]);
    }
}
