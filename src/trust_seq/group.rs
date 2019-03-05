use std::cmp;

#[derive(Serialize)]
pub struct BaseGroup {
    pub lower_count: usize,
    pub upper_count: usize,
}
#[derive(Clone, Debug)]
pub enum GroupType {
    None,
    linear,
    exponential,
}
impl BaseGroup {
    fn make_ungrouped_groups(max_len: usize) -> Vec<BaseGroup> {
        let mut v = Vec::new();
        for base in 1..(max_len + 1) {
            v.push(BaseGroup {
                lower_count: base,
                upper_count: base,
            });
        }
        return v;
    }
    fn calc_interval(max_len: usize) -> usize {
        let base_values = [2, 5, 10];
        let mut multiplier = 1;
        loop {
            for base_value in base_values.iter() {
                let interval = *base_value * multiplier;
                let group_count = (max_len as f64 / interval as f64).ceil() as usize;
                if group_count < 66 {
                    return interval;
                }
            }
            multiplier *= 10;
        }
    }
    fn make_linear_groups(max_len: usize) -> Vec<BaseGroup> {
        if max_len <= 75 {
            return BaseGroup::make_ungrouped_groups(max_len);
        }
        let mut v = BaseGroup::make_ungrouped_groups(9);
        let interval = BaseGroup::calc_interval(max_len - 9);
        let mut start_pos = 10;
        while start_pos <= max_len {
            let end_pos = cmp::min(start_pos + interval - 1, max_len);
            v.push(BaseGroup {
                lower_count: start_pos,
                upper_count: end_pos,
            });
            start_pos += interval;
        }
        return v;
    }
    fn make_exponential_groups(max_len: usize) -> Vec<BaseGroup> {
        let mut start_pos = 1;
        let mut interval = 1;
        let mut v: Vec<BaseGroup> = Vec::new();
        while start_pos <= max_len {
            let end_pos = cmp::min(start_pos + interval - 1, max_len);
            v.push(BaseGroup {
                lower_count: start_pos,
                upper_count: end_pos,
            });
            start_pos += interval;
            if start_pos == 10 && max_len > 75 {
                interval = 5;
            }
            if start_pos == 50 && max_len > 200 {
                interval = 10;
            }
            if start_pos == 100 && max_len > 300 {
                interval = 50;
            }
            if start_pos == 500 && max_len > 1000 {
                interval = 100;
            }
            if start_pos == 1000 && max_len > 2000 {
                interval = 500;
            }
        }
        return v;
    }
    pub fn make_base_groups(group_type: &GroupType, max_len: usize) -> Vec<BaseGroup> {
        match *group_type {
            GroupType::None => BaseGroup::make_ungrouped_groups(max_len),
            GroupType::linear => BaseGroup::make_linear_groups(max_len),
            GroupType::exponential => BaseGroup::make_exponential_groups(max_len),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::BaseGroup;
    use super::GroupType;
    fn check_base_groups(group_type: GroupType, max_len: usize, expected_len: usize) -> () {
        let groups = BaseGroup::make_base_groups(&group_type, max_len);
        let mut prev_end = 0;
        for group in &groups {
            assert_eq!(prev_end + 1, group.lower_count);
            prev_end = group.upper_count;
        }
        assert_eq!(expected_len, groups.len());
    }
    #[test]
    fn test_linear_group() {
        check_base_groups(GroupType::linear, 70, 70);
        check_base_groups(GroupType::linear, 75, 75);
        check_base_groups(GroupType::linear, 76, 43);
        check_base_groups(GroupType::linear, 100, 55);
        check_base_groups(GroupType::linear, 139, 74);
        check_base_groups(GroupType::linear, 140, 36);
        check_base_groups(GroupType::linear, 500, 59);
    }
}
