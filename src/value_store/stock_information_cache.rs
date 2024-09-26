use std::collections::HashMap;
use std::collections::VecDeque;

use crate::value_store::stock_information::StockInformation;
use crate::value_store::technical_analysis::TechnicalAnalysis;

pub struct StockInformationCache {
    stock_info_map: HashMap<String, StockInformation>,
    stock_history_map: HashMap<String, HashMap<usize, VecDeque<StockInformation>>>,
    technical_analysis: TechnicalAnalysis,
}

impl StockInformationCache {
    pub fn new() -> Self {
        StockInformationCache { 
            stock_info_map:HashMap::new(), 
            stock_history_map:HashMap::new(),
            technical_analysis: TechnicalAnalysis::new(120.0),
        }
    }

    pub fn add_json(&mut self, json_data: &str) -> (String, usize, i64, String) {
        let stock_info:StockInformation = parse_json_to_stock_info(json_data);

        if stock_info.volume_moved != 0 && stock_info.stock_interval == 1 || !self.stock_info_map.contains_key(&stock_info.stock_name) {
            self.stock_info_map.insert(stock_info.stock_name.clone(), stock_info.clone());
        }

        let key:(String, usize) = (stock_info.stock_name.clone(), stock_info.stock_interval);

        let interval_map = match self.stock_history_map.get_mut(&stock_info.stock_name) {
            Some(v) => v,
            None => {
                self.stock_history_map.insert(stock_info.stock_name.clone(), HashMap::new());

                self.stock_history_map.get_mut(&stock_info.stock_name).unwrap()
            }
        };

        let stock_history = match interval_map.get_mut(&stock_info.stock_interval) {
            Some(v) => v,
            None => {
                interval_map.insert(stock_info.stock_interval, VecDeque::new());

                interval_map.get_mut(&stock_info.stock_interval).unwrap()
            }
        };

        if stock_history.len() >= 120 { 
            let removed_stock = match stock_history.pop_front() {
                Some(v) => v,
                None => StockInformation::new(),
            };

            self.technical_analysis.remove_trade(&key, &stock_info);
        }

        self.technical_analysis.add_trade(&key, &stock_info);

        let stock_info_string = stock_info.to_string();
        let volume_moved = stock_info.volume_moved;

        stock_history.push_back(stock_info);

        (key.0, key.1, volume_moved, stock_info_string)
    }

    pub fn get_vec_dashboard(&self) -> Vec<String> {
        let mut string_pair:Vec<(String, String)> = self.stock_info_map.values()
                    .map(|stock_info| (stock_info.stock_name.clone(), stock_info.to_string()))
                    .collect();

        string_pair.sort_by(|a, b| a.0.cmp(&b.0));

        string_pair.into_iter().map(|stock_tuple| stock_tuple.1).collect()
    }

    pub fn get_vec_of_stock(&self, stock: &String) -> Vec<String> {
        let mut stock_list = Vec::<String>::new();

        match self.stock_history_map.get(stock) {
            Some(v) => {
                for stock_history in v.values().into_iter() {
                    for stock_info in stock_history.iter() {
                        stock_list.push(stock_info.to_string());
                    }
                }
                
                stock_list.push("End of update".to_string());
            },
            None => (),
        }

        stock_list
    }

    pub fn has_key(&self, stock: &String) -> bool {
        self.stock_history_map.contains_key(stock)
    }
}

pub fn parse_json_to_stock_info(json_data: &str) -> StockInformation {
    let mut tmp: String = String::new();
    let mut key: String = String::new();
    let mut stock_info = StockInformation::new();

    for p in json_data.chars() {
        match p {
            ' ' | '\n' | '\t' | '\"' | '{' | '}' => (),
            ':' | ',' => {
                match key.len() {
                    0 => key = tmp,
                    _ => {
                        stock_info.insert_data(key, tmp);
                        key = String::new();
                    }
                }
                
                tmp = String::new();
            },
            _ => tmp.push(p),
        };
    }

    stock_info.insert_data(key, tmp);

    stock_info
}