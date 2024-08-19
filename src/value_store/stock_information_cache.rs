use std::collections::HashMap;
use std::collections::VecDeque;

pub struct StockInformation {
    pub stock_name: String,
    pub stock_interval: usize,
    pub timestamp:i64,

    pub avg_price: f64,
    pub avg_price_open: f64,
    pub min_price: f64,
    pub max_price: f64,

    pub volume_moved: i64,
    pub num_of_trades: i64,
}

impl StockInformation {
    pub fn new() -> Self {
        StockInformation { 
            stock_name: String::new(),
            stock_interval: 0,
            timestamp: 0,
            avg_price: 0.0,
            avg_price_open: 0.0,
            min_price: 0.0,
            max_price: 0.0,
            volume_moved: 0,
            num_of_trades: 0
        }
    }

    pub fn clone(&self) -> Self {
        StockInformation {
            stock_name: self.stock_name.clone(),
            stock_interval: self.stock_interval,
            timestamp: self.timestamp,
            avg_price: self.avg_price,
            avg_price_open: self.avg_price_open,
            min_price: self.min_price,
            max_price: self.max_price,
            volume_moved: self.volume_moved,
            num_of_trades: self.num_of_trades
        }
    }

    pub fn insert_data(&mut self, key: String, value: String) {
        match key.as_str() {
            "sn" => self.stock_name = value,
            "si" => self.stock_interval = value.parse::<usize>().unwrap(),
            "ap" => self.avg_price = value.parse::<f64>().unwrap(),
            "op" => self.avg_price_open = value.parse::<f64>().unwrap(),
            "mn" => self.min_price = value.parse::<f64>().unwrap(),
            "mx" => self.max_price = value.parse::<f64>().unwrap(),
            "vm" => self.volume_moved = value.parse::<i64>().unwrap(),
            "nt" => self.num_of_trades = value.parse::<i64>().unwrap(),
            "t" => self.timestamp = value.parse::<i64>().unwrap(),
            _ => (),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{{
            \"stock_interval\": {},
            \"name\": \"{}\",
            \"avg_price\": \"{:.6}\",
            \"avg_price_open\": {:.6},
            \"min_price\": {},
            \"max_price\": {},
            \"volume_moved\": {},
            \"num_of_trades\": {},
            \"timestamp\": {}
        }}",
            self.stock_interval,
            self.stock_name,
            self.avg_price as f64,
            self.avg_price_open as f64,
            self.min_price as f64,
            self.max_price as f64,
            self.volume_moved,
            self.num_of_trades,
            self.timestamp,
        )
    }
}

pub struct StockInformationCache {
    stock_info_map: HashMap<String, StockInformation>,
    stock_history_map: HashMap<(String, usize), VecDeque<StockInformation>>,
}

impl StockInformationCache {
    pub fn new() -> Self {
        StockInformationCache{ stock_info_map:HashMap::new(), stock_history_map:HashMap::new() }
    }

    pub fn add_json(&mut self, json_data: &str) -> (String, usize, i64, String) {
        let stock_info:StockInformation = parse_json_to_stock_info(json_data);

        self.stock_info_map.insert(stock_info.stock_name.clone(), stock_info.clone());

        let key:(String, usize) = (stock_info.stock_name.clone(), stock_info.stock_interval);

        if !self.stock_history_map.contains_key(&key) {
            self.stock_history_map.insert(key.clone(), VecDeque::new());
        }

        let stock_history = self.stock_history_map.get_mut(&key).unwrap();

        if stock_history.len() > 120 { 
            stock_history.pop_front(); 
        }

        stock_history.push_back(stock_info.clone());

        (key.0, key.1, stock_info.volume_moved, stock_info.to_string())
    }

    pub fn get_vec_dashboard(&self) -> Vec<String> {
        let mut string_pair:Vec<(String, String)> = self.stock_info_map.values()
                    .map(|stock_info| (stock_info.stock_name.clone(), stock_info.to_string()))
                    .collect();

        string_pair.sort_by(|a, b| a.0.cmp(&b.0));

        string_pair.into_iter().map(|stock_tuple| stock_tuple.1).collect()
    }

    pub fn get_vec_of_stock(&self, key: &(String, usize)) -> Vec<String> {
        match self.stock_history_map.get(key) {
            Some(v) => {
                v.into_iter().map(|stock_info| stock_info.to_string()).collect()
            },
            None => Vec::new(),
        }
    }

    pub fn has_key(&self, key: &(String, usize)) -> bool {
        self.stock_history_map.contains_key(key)
    }
}

pub fn parse_json_to_stock_info(json_data: &str) -> StockInformation {
    let mut tmp: String = String::new();
    let mut key: String = String::new();
    let mut stock_info = StockInformation::new();

    for p in json_data.chars() {
        if p == ' ' || p == '\n' || p == '\t' || p == '\"' || p == '{' || p == '}' { 
            continue; 
        }
        
        if p == ':' || p == ',' {
            match key.len() {
                0 => key = tmp,
                _ => {
                    stock_info.insert_data(key, tmp);
                    key = String::new();
                }
            }
            
            tmp = String::new();
            
            continue;
        }

        tmp.push(p);
    }

    stock_info.insert_data(key, tmp);

    stock_info
}