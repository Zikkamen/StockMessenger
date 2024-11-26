use std::{
    collections::{HashSet, HashMap},
    sync::{Arc, RwLock},
};

use crate::value_store::{StockInformationCacheInterface, OHLCModel};

#[derive(Clone)]
pub struct ConnectionService {
    stock_cache: StockInformationCacheInterface,
    conn_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    subscr_map: Arc<RwLock<HashMap::<String, HashSet<usize>>>>,
}

impl ConnectionService {
    pub fn new() -> Self {
        ConnectionService {
            stock_cache: StockInformationCacheInterface::new(),
            conn_queue: Arc::new(RwLock::new(HashMap::new())),
            subscr_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_ohlc_json(&self, json_data: String) -> OHLCModel {
        self.stock_cache.add_json(json_data)
    }

    pub fn get_subscribers(&self, stock_name: &String) -> HashSet<usize> {
        match self.subscr_map.read().unwrap().get(stock_name) {
            Some(v) => v.clone(),
            None => HashSet::new(),
        }
    }

    pub fn add_events(&self, ids_to_update: HashSet<usize>, event: String) {
        let mut connection_vec = self.conn_queue.write().unwrap();

        for id in ids_to_update.iter() {
            match connection_vec.get_mut(id) {
                Some(v) => {
                    if v.len() < 1000 {
                        v.push(event.clone());
                    }
                },
                None => continue,
            };
        }
    }

    pub fn read_events(&self, id: &usize) -> Vec<String> {
        match self.conn_queue.write().unwrap().get_mut(id) {
            Some(v) => {
                let conn_vec = v.clone();
                *v = Vec::<String>::new();

                conn_vec
            },
            None => Vec::new(),
        }

    }

    pub fn remove_stock_subscription(&self, id: usize, stock_name: &String) {
        if stock_name.len() == 0 {
            return;
        }

        match self.subscr_map.write().unwrap().get_mut(stock_name) {
            Some(v) => { v.remove(&id); },
            None => println!("Couldn't find key {:?}", stock_name),
        };
    }

    pub fn add_stock_subscription(&self, id: usize, stock_name: &String) {
        if !self.stock_cache.has_key(stock_name) {
            println!("Couldn't find key stock_name {:?}", stock_name);

            return;
        }

        let mut subscr_map = self.subscr_map.write().unwrap();

        match subscr_map.get_mut(stock_name) {
            Some(v) => {
                v.insert(id);
            },
            None => {
                subscr_map.insert(stock_name.clone(), HashSet::from([id]));
            }
        };

        self.conn_queue.write().unwrap().insert(id, self.stock_cache.get_vec_of_stock(stock_name));
    }

    pub fn add_subscriber(&self) -> usize {
        let mut conn_queue = self.conn_queue.write().unwrap();
        let n = conn_queue.len();
        conn_queue.insert(n, Vec::new());

        n
    }

    pub fn remove_subscriber(&self, id: usize) {
        self.conn_queue.write().unwrap().remove(&id);
    }
}