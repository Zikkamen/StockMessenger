use std::thread;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::collections::{HashSet, HashMap};

use tungstenite::{
    connect,
    Message,
};

use crate::value_store::stock_information_cache::StockInformationCache;

pub struct NotificationClient {
    ip_client: String,
    connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    subscriber_map: Arc<RwLock<HashMap::<String, HashSet<usize>>>>,
    stock_information_cache: Arc<RwLock<StockInformationCache>>,
}

impl NotificationClient {
    pub fn new(ip_client: String,
               connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
               subscriber_map: Arc<RwLock<HashMap::<String, HashSet<usize>>>>,
               stock_information_cache: Arc<RwLock<StockInformationCache>>) -> Self {
        NotificationClient {
            ip_client: ip_client, 
            connection_queue: connection_queue, 
            subscriber_map: subscriber_map,
            stock_information_cache: stock_information_cache,
        }
    }

    pub fn start_client(&mut self) {
        loop {
            println!("Trying to Connect");
    
            let (mut client, _response) = match connect(format!("ws://{}", self.ip_client)) {
                Ok(v) => v,
                Err(_v) => { 
                    thread::sleep(Duration::from_millis(1000)); 
                    
                    continue;
                },
            };

            let _ = client.send(Message::Text("{stock: *}".to_string()));
    
            loop {
                let message = match client.read() {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Error receiving message {} \n Closing Client", e);
                        break;
                    },
                };
    
                match message {
                    msg @ Message::Text(_) => {
                        let text: String = msg.into_text().unwrap();
                        let (name, interval, volume_moved, json) = self.stock_information_cache.write().unwrap().add_json(&text);

                        let mut ids_to_update:HashSet<usize> = HashSet::new();

                        match self.subscriber_map.read().unwrap().get(&name){
                            Some(list_of_ids) => {
                                for id in list_of_ids.iter() {
                                    ids_to_update.insert(*id);
                                }
                            },
                            None => (),
                        }
                        
                        if interval == 1 && volume_moved > 0 {
                            match self.subscriber_map.read().unwrap().get("*"){
                                Some(list_of_ids) => {
                                    for id in list_of_ids.iter() {
                                        ids_to_update.insert(*id);
                                    }
                                },
                                None => (),
                            }
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