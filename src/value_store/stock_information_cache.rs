use std::{
    sync::{Arc, RwLock},
    collections::{HashMap, VecDeque}
};

use crate::value_store::OHLCModel;

struct StockInformation {
    stock_history: [VecDeque<OHLCModel>; 5],
}

impl StockInformation {
    pub fn new() -> Self {
        StockInformation {
            stock_history: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new()
            ],
        }
    }

    pub fn add_ohlc(&mut self, ohlc_model: OHLCModel) {
        let id = match ohlc_model.stock_interval {
            1 => 0,
            10 => 1,
            60 => 2,
            300 => 3,
            600 => 4,
            _ => return,
        };

        self.stock_history[id].push_back(ohlc_model);

        if self.stock_history[id].len() > 120 {
            let _ = self.stock_history[id].pop_front();
        }
    }
}

struct StockInformationCache {
    stock_map: HashMap<String, usize>,
    stock_vec: Vec<StockInformation>,
}

impl StockInformationCache {
    pub fn new() -> Self {
        StockInformationCache { 
            stock_map: HashMap::new(), 
            stock_vec: Vec::new(),
        }
    }

    pub fn add_json(&mut self, json_data: String) -> OHLCModel {
        let mut last_ohlc_mode = OHLCModel::new();

        for ohlc_model in parse_ohlc_models(json_data).into_iter() {
            let id = match self.stock_map.get(&ohlc_model.stock_name) {
                Some(v) => *v,
                None => {
                    let n = self.stock_vec.len();
    
                    self.stock_vec.push(StockInformation::new());
                    self.stock_map.insert(ohlc_model.stock_name.clone(), n);
    
                    n
                }
            };
    
            self.stock_vec[id].add_ohlc(ohlc_model.clone());
            last_ohlc_mode = ohlc_model;
        }

        last_ohlc_mode
    }

    pub fn has_key(&self, name: &String) -> bool {
        self.stock_map.contains_key(name)
    }

    pub fn get_vec_of_stock(&self, name: &String) -> Vec<String> {
        let id = match self.stock_map.get(name) {
            Some(v) => *v,
            None => return Vec::new(),
        };

        let mut stock_vec = Vec::<String>::new();

        for i in 0..5 {
            for stock in self.stock_vec[id].stock_history[i].iter() {
                stock_vec.push(stock.to_string());
            }   
        }

        stock_vec.push("End of Update".to_owned());

        stock_vec
    }
}

#[derive(Clone)]
pub struct StockInformationCacheInterface {
    stock_cache: Arc<RwLock<StockInformationCache>>,
}

impl StockInformationCacheInterface {
    pub fn new() -> Self {
        StockInformationCacheInterface {
            stock_cache: Arc::new(RwLock::new(StockInformationCache::new())),
        }
    }

    pub fn add_json(&self, json_data:String) -> OHLCModel {
        self.stock_cache.write().unwrap().add_json(json_data)
    }

    pub fn has_key(&self, name: &String) -> bool {
        self.stock_cache.read().unwrap().has_key(name)
    }

    pub fn get_vec_of_stock(&self, name: &String) -> Vec<String> {
        self.stock_cache.read().unwrap().get_vec_of_stock(name)
    }
}

fn parse_ohlc_models(json_data: String) -> Vec<OHLCModel> {
    let mut ohlc_models = Vec::<OHLCModel>::new();
    let mut tmp = String::new();

    for c in json_data.chars() {
        tmp.push(c);

        if c == '\n' {
            ohlc_models.push(OHLCModel::from_string(tmp));

            tmp = String::new();
        }
    }

    ohlc_models
}
