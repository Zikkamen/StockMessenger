use std::thread;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::collections::HashSet;

use websocket::ClientBuilder;
use websocket::{OwnedMessage};
use websocket::server::upgrade::WsUpgrade;

use crate::value_store::stock_information_cache::StockInformationCache;

pub struct NotificationClient {
    connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    subscriber_map: Arc<RwLock<HashMap::<(String, String), Vec<usize>>>>,
    stock_information_cache: Arc<RwLock<StockInformationCache>>,
}

impl NotificationClient {
    pub fn new(connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
               subscriber_map: Arc<RwLock<HashMap::<(String, String), Vec<usize>>>>,
               stock_information_cache: Arc<RwLock<StockInformationCache>>) -> Self {
        NotificationClient{ 
            connection_queue: connection_queue, 
            subscriber_map: subscriber_map,
            stock_information_cache: stock_information_cache,
        }
    }

    pub fn start_client(&mut self) {
        loop {
            println!("Trying to Connect");
    
            let mut client = match ClientBuilder::new("ws://localhost:9004").unwrap().connect_insecure() {
                Ok(v) => v,
                Err(v) => { thread::sleep(Duration::from_millis(1000)); continue },
            };
    
            loop {
                let message:OwnedMessage = match client.recv_message() {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Error receiving message {} \n Closing Client", e);
                        break;
                    },
                };
    
                match message {
                    OwnedMessage::Text(txt) => {
                        let text: String = txt.parse().unwrap();
                        let (name, interval, json) = self.stock_information_cache.write().unwrap().add_json(&text);

                        let mut ids_to_update:HashSet<usize> = HashSet::new();

                        match self.subscriber_map.read().unwrap().get(&("stock".to_string(), name.clone())){
                            Some(list_of_ids) => {
                                for id in list_of_ids.iter() {
                                    ids_to_update.insert(*id);
                                }
                            },
                            None => (),
                        }
                        
                        match self.subscriber_map.read().unwrap().get(&("interval".to_string(), interval.to_string())){
                            Some(list_of_ids) => {
                                for id in list_of_ids.iter() {
                                    ids_to_update.insert(*id);
                                }
                            },
                            None => (),
                        }

                        let mut connection_vec = self.connection_queue.write().unwrap();

                        for id in ids_to_update.iter() {
                            match connection_vec.get_mut(id) {
                                Some(v) => v.push(json.clone()),
                                None => continue,
                            };
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}