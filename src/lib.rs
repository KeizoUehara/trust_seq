#[macro_use]
extern crate serde_derive;

extern crate getopts;
extern crate serde;
extern crate serde_json;
mod trust_seq;

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use trust_seq::contaminant::find_contaminant;
    use trust_seq::contaminant::Contaminant;
    use trust_seq::contaminant_list::CONTAMINANT_LIST;
    #[test]
    fn test_load_contaminants() {
        let cons = Contaminant::load_contaminants(BufReader::new(CONTAMINANT_LIST.as_bytes()));
        let hit = find_contaminant(&cons, "ACACTCTTTCCCTACACGACGCTCTTCCGATCT".as_bytes());
        println!(
            "query = {} , hit={:?}",
            "ACACTCTTTCCCTACACGACGCTCTTCCGATCT", hit
        );
        assert_eq!(151, cons.len());
        println!("{:?}", cons[0]);
    }
}
