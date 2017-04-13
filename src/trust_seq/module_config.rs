use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use super::limits;
use std::io::Error;
use super::utils::split_by_space;

#[derive(Debug,Clone)]
pub struct ModuleConfig {
    params: HashMap<String, f64>,
}
impl ModuleConfig {
    pub fn new() -> ModuleConfig {
        let mut config = ModuleConfig { params: HashMap::new() };
        config
            .load(BufReader::new(limits::LIMIT_DEFAULT.as_bytes()))
            .unwrap();
        return config;
    }
    pub fn get(&self, key: &str) -> f64 {
        return *self.params.get(key).unwrap();
    }
    pub fn load<R: BufRead>(&mut self, reader: R) -> Result<(), Error> {
        for rlst in reader.lines() {
            let line = rlst?;
            if line.starts_with('#') || line.len() == 0 {
                continue;
            }
            let vals = split_by_space(&line);
            if vals.len() != 3 {
                println!("Config line '{}' didn't contain the 3 required sections",
                         &line);
                break;
            }
            let rslt = vals[2].parse::<f64>();
            if let Ok(val) = rslt {
                self.params
                    .insert(format!("{}:{}", vals[0], vals[1]), val);
            }
        }
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::ModuleConfig;

    #[test]
    fn test_module_config() {
        let module_config = ModuleConfig::new();
        assert_eq!(module_config.get("duplication:warn"), 70.0);
        assert_eq!(module_config.get("duplication:error"), 50.0);
    }
}
