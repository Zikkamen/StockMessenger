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
            _ => println!("Error key not found {}", key),
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
    stock_info_map: HashMap<usize, HashMap<String, StockInformation>>,
    stock_history_map: HashMap<String, VecDeque<StockInformation>>,
}

impl StockInformationCache {
    pub fn new() -> Self {
        StockInformationCache{ stock_info_map:HashMap::new(), stock_history_map:HashMap::new() }
    }

    pub fn add_json(&mut self, json_data: &str) -> (String, usize, String) {
        let mut tmp: String = String::new();
        let mut key: String = String::new();

        let mut stock_info = StockInformation::new();

        for p in json_data.chars() {
            if p == ' ' || p == '\n' || p == '\t' || p == '\"' || p == '{' || p == '}' { 
                continue; 
            }
            
            if p == ':' || p == ',' {
                if key.len() == 0 {
                    key = tmp;
                } else {
                    stock_info.insert_data(key, tmp);
                    key = String::new();
                }
                
                tmp = String::new();
                
                continue;
            }

            tmp.push(p);
        }

        stock_info.insert_data(key, tmp);

        let stock_info_json = stock_info.to_string();
        let stock_info_interval = stock_info.stock_interval;
        let stock_info_name = stock_info.stock_name.clone();

        if !self.stock_info_map.contains_key(&stock_info.stock_interval) {
            self.stock_info_map.insert(stock_info.stock_interval, HashMap::new());
        }

        self.stock_info_map.get_mut(&stock_info.stock_interval)
            .unwrap()
            .insert(stock_info.stock_name.clone(), stock_info.clone());

        if !self.stock_history_map.contains_key(&stock_info.stock_name) {
            self.stock_history_map.insert(stock_info.stock_name.clone(), VecDeque::new());
        }

        let stock_history = self.stock_history_map.get_mut(&stock_info.stock_name).unwrap();

        if stock_info.stock_interval == 1{
            if stock_history.len() > 60 { stock_history.pop_front(); }

            stock_history.push_back(stock_info);
        }

        (stock_info_name, stock_info_interval, stock_info_json)
    }

    pub fn get_vec_of_interval(&self, stock_interval: usize) -> Vec<String> {
        match self.stock_info_map.get(&stock_interval) {
            Some(v) => {
                let mut string_pair:Vec<(String, String)> = v.values()
                    .map(|stock_info| (stock_info.stock_name.clone(), stock_info.to_string()))
                    .collect();
                string_pair.sort_by(|a, b| a.0.cmp(&b.0));

                string_pair.into_iter().map(|stock_tuple| stock_tuple.1).collect()
            },
            None => Vec::new(),
        }
    }

    pub fn get_vec_of_stock(&self, stock_name: String) -> Vec<String> {
        match self.stock_history_map.get(&stock_name) {
            Some(v) => {
                v.into_iter().map(|stock_info| stock_info.to_string()).collect()
            },
            None => Vec::new(),
        }
    }
}