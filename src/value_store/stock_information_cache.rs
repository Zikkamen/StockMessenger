use crate::HashMap;

pub struct StockInformationCache {
    stock_info_map: HashMap<String, String>,
}

impl StockInformationCache {
    pub fn new() -> Self {
        StockInformationCache{ stock_info_map:HashMap::new() }
    }

    pub fn add_json(&mut self, json_data: &str) {
        let mut tmp: String = String::new();
        let mut prev_is_name:bool = false;

        for p in json_data.chars() {
            if p == ' ' || p == '\n' || p == '\t' || p == '\"' || p == '{' || p == '}' { 
                continue; 
            }
            
            if p == ':' || p == ',' {
                if prev_is_name { break; }
                if tmp == "name"{ prev_is_name = true; }

                tmp.clear();
                continue;
            }

            tmp.push(p);
        }

        if tmp.len() > 0 {
            self.stock_info_map.insert(tmp, json_data.to_string());
        }
    }

    pub fn get_vec_of_cache(&self) -> Vec<String> {
        let mut tuple_list:Vec<(String, String)> = self.stock_info_map.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect();

        tuple_list.sort_by(|(a1, a2), (b1, b2)| a1.cmp(&b1));
        tuple_list.into_iter().map(|(key, value)| value).collect()
    }
}