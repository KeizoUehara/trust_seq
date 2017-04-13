pub struct BaseGroup {
    lower_count: usize,
    upper_count: usize,
}
pub enum GroupType{
    none,
    linear,
    exponential
}
impl BaseGroup{
    fn make_ungrouped_groups(max_len:usize) -> Vec<BaseGroup>{
        let v = Vec::new();
        return v;
    }
    fn make_base_groups(group_type:GroupType,max_len:usize){
        match group_type {
            none => 
        };
    }
}
