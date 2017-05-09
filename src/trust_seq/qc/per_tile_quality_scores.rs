use std::cmp;

use std::f64;
use std::i32;
use std::str;
use std::str::FromStr;
use std::io::Write;
use std::collections::HashMap;
use serde_json::value::Value;
use serde_json::value;
use serde_json::map::Map;
use trust_seq::utils::Sequence;
use trust_seq::trust_seq::{TrustSeqConfig, TrustSeqErr};
use trust_seq::qc::{QCModule,QCResult,QCReport};
use trust_seq::qc::PhreadEncoding;
use trust_seq::qc::quality_counts::QualityCounts;
use trust_seq::group::BaseGroup;

pub struct PerTileQualityScores<'a> {
    ignore_in_report: bool,
    total_count: u64,
    id_position: i32,
    current_length: usize,
    min_char: u8,
    max_char: u8,
    quality_counts: HashMap<u32, QualityCounts>,
    config: &'a TrustSeqConfig,
}

#[derive(Serialize)]
struct PerTileQualityReport {
    status: QCResult,
    tiles: Vec<u32>,
    groups: Vec<BaseGroup>,
    qualities: Vec<Vec<f64>>
}
impl<'a> PerTileQualityScores<'a> {
    pub fn new(config: &'a TrustSeqConfig) -> PerTileQualityScores {
        return PerTileQualityScores {
            total_count:0,
            ignore_in_report: false,
            id_position: -1,
            quality_counts: HashMap::new(),
            min_char: 255,
            max_char: 0,
            current_length: 0,
            config: config,
        };
    }
}
impl QCReport for PerTileQualityReport{
    fn get_name(&self) -> &'static str {
        return "Per tile sequence quality";
    }
    fn get_status(&self) -> QCResult{
        return self.status;
    }
    fn add_json(&self,map:&mut Map<String,Value>) -> Result<(), TrustSeqErr> {
        map.insert(self.get_name().to_string(), value::to_value(&self)?);
        return Ok(());
    }
    fn print_text_report(&self, writer: &mut Write) -> Result<(), TrustSeqErr> {
        write!(writer,"Tile\tBase\tMean\n")?;
        for (idx,tile) in self.tiles.iter().enumerate(){
            for (group_idx,group) in self.groups.iter().enumerate(){
                if group.lower_count == group.upper_count {
                    writeln!(writer,"{}\t{}\t{}",tile,group.lower_count,self.qualities[idx][group_idx])?;
                }else{
                    writeln!(writer,"{}\t{}-{}\t{}",tile,group.lower_count,group.upper_count,self.qualities[idx][group_idx])?;
                }
            }
        }
        return Ok(());
    }
}
impl<'a> QCModule for PerTileQualityScores<'a> {
    fn process_sequence(&mut self, seq: &Sequence) -> () {
        if self.ignore_in_report {
            return;
        }
        self.total_count += 1;
        if self.total_count % 10 != 0 {
            return;
        }
        let id_str = unsafe {
            str::from_utf8_unchecked(seq.id)
        };
        let split_ids: Vec<&str> = id_str.split(":").collect();
        if self.id_position < 0 {
            if split_ids.len() >= 7 {
                self.id_position = 4;
            } else if split_ids.len() >= 5 {
                self.id_position = 2;
            } else {
                self.ignore_in_report = true;
                return;
            }
        } 
        let tile: i32;
        match i32::from_str(split_ids[self.id_position as usize]) {
            Ok(v) => tile = v,
            Err(_) => {
                self.ignore_in_report = true;
                return;
            }
        };
        if !self.quality_counts.contains_key(&(tile as u32)) {
            if self.quality_counts.len() > 500{
                println!("Too many tiles (>500) so giving up trying to do per-tile qualities since we're probably parsing the file wrongly");
                self.ignore_in_report = true;
                return;
            }
            self.quality_counts.insert(tile as u32,QualityCounts::new());
        }
        let q = self.quality_counts.get_mut(&(tile as u32)).unwrap();
        q.ensure_size(seq.quality.len());
        self.current_length = cmp::max(self.current_length,seq.quality.len());
        for (idx, ch) in seq.quality.iter().enumerate() {
            self.min_char = cmp::min(self.min_char, *ch);
            self.max_char = cmp::max(self.max_char, *ch);
            
            q.add_value(idx,*ch);
        }
    }
    fn ignore_in_report(&self) -> bool{
        return self.ignore_in_report;
    }
    fn calculate(&self,reports:&mut Vec<Box<QCReport>>) -> Result<(), TrustSeqErr>{
        if self.ignore_in_report{
            return Ok(());
        }
        let encode = PhreadEncoding::get_phread_encoding(self.min_char).unwrap();
        let offset = encode.offset as u32;
        let groups = BaseGroup::make_base_groups(&self.config.group_type, self.current_length);
        let mut tile_numbers : Vec<u32> = self.quality_counts.keys().map(|x:&u32| *x).collect();
        tile_numbers.sort();
        let mut means : Vec<Vec<f64>> = Vec::new();
        means.resize(tile_numbers.len(),Vec::new());
        let mut average_qualities_per_group:Vec<f64> = Vec::new();
        average_qualities_per_group.resize(groups.len(),0.0);
        for (t_idx,tile) in tile_numbers.iter().enumerate(){
            let mut mean:Vec<f64> = Vec::new();
            let q_c = self.quality_counts.get(tile).unwrap();
            mean.resize(groups.len(),0.0);
            for (idx,group) in groups.iter().enumerate() {
                mean[idx] = q_c.get_mean(group,offset);
                average_qualities_per_group[idx] += mean[idx];
            }
            means[t_idx] = mean;
        }

        for v in &mut average_qualities_per_group{
            *v /= tile_numbers.len() as f64;
        }
        let mut max_deviation:f64 = 0.0;
        for idx in 0..groups.len(){
            for mean in &mut means{
                mean[idx] -= average_qualities_per_group[idx];
                max_deviation = max_deviation.max(mean[idx]);
            }
        }
        let status = if max_deviation > self.config.module_config.get("tile:error"){
            QCResult::Fail
        }else if  max_deviation > self.config.module_config.get("tile:warn"){
            QCResult::Warn
        }else{
            QCResult::Pass
        };
        reports.push(Box::new(PerTileQualityReport{
            status: status,
            tiles: tile_numbers,
            groups: groups,
            qualities: means
        }));
        return Ok(());
    }
}
