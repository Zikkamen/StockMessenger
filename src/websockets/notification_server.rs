use std::thread;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::collections::HashSet;

use websocket::sync::Server;
use websocket::{OwnedMessage};
use websocket::sync::Writer;
use websocket::server::upgrade::WsUpgrade;
use websocket::stream::sync::TcpStream;

use crate::value_store::stock_information_cache::StockInformationCache;

pub struct NotificationServer {
    connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    subscriber_map: Arc<RwLock<HashMap::<(String, String), HashSet<usize>>>>,
    stock_information_cache: Arc<RwLock<StockInformationCache>>,
}

impl NotificationServer {
    pub fn new(connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
               subscriber_map: Arc<RwLock<HashMap::<(String, String), HashSet<usize>>>>,
               stock_information_cache: Arc<RwLock<StockInformationCache>>) -> Self {
        NotificationServer{ 
            connection_queue: connection_queue,
            subscriber_map: subscriber_map,
            stock_information_cache: stock_information_cache,
        }
    }

    pub fn start_server(&self) {
        start_websocketserver(Arc::clone(&self.connection_queue), Arc::clone(&self.subscriber_map), Arc::clone(&self.stock_information_cache));
    }
}

fn start_websocketserver(connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
                         subscriber_map: Arc<RwLock<HashMap::<(String, String), HashSet<usize>>>>,
                         stock_information_cache: Arc<RwLock<StockInformationCache>>){
    let server = Server::bind("127.0.0.1:9002").unwrap();

    thread::spawn(move || {
        let mut id:usize = 0;

        for connection in server.filter_map(Result::ok) {
            let client = connection.accept().unwrap();
            let (mut receiver, mut sender) = client.split().unwrap();

            let message_json:String = match receiver.recv_message() {
                Ok(p) => read_message(p),
                Err(_e) => {
                    return;
                }
            };

            let parsed_json:(String, String) = parse_json(&message_json);

            if &parsed_json.0 == "stock" {
                let key_stock = ("stock".to_string(), parsed_json.1.clone());

                if !subscriber_map.read().unwrap().contains_key(&key_stock) {
                    subscriber_map.write().unwrap().insert(key_stock.clone(), HashSet::from([id]));
                } else {
                    subscriber_map.write().unwrap().get_mut(&key_stock).unwrap().insert(id);
                }

                connection_queue.write().unwrap().insert(id, stock_information_cache.read().unwrap().get_vec_of_stock(parsed_json.1));
                start_websocket(sender, Arc::clone(&connection_queue), Arc::clone(&subscriber_map), key_stock, id);
            } else if &parsed_json.0 == "interval" {
                let key_stock = ("interval".to_string(), parsed_json.1.clone());

                if !subscriber_map.read().unwrap().contains_key(&key_stock) {
                    subscriber_map.write().unwrap().insert(key_stock.clone(), HashSet::from([id]));
                } else {
                    subscriber_map.write().unwrap().get_mut(&key_stock).unwrap().insert(id);
                }

                connection_queue.write().unwrap().insert(id, stock_information_cache.read().unwrap().get_vec_of_interval(parsed_json.1.parse::<usize>().unwrap()));
                start_websocket(sender, Arc::clone(&connection_queue), Arc::clone(&subscriber_map), key_stock, id);
            }

            println!("Spawned websocket {}", id);
            id += 1;
        }
    });
}

fn start_websocket(mut sender: Writer<TcpStream>,
                   connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
                   subscriber_map: Arc<RwLock<HashMap::<(String, String), HashSet<usize>>>>,
                   key: (String, String),
                   id: usize) {
    thread::spawn(move || {
        let thread_id = id;
        let mut ping_cnt:usize = 0;

        loop {
            if connection_queue.read().unwrap().len() > 100 { continue; }

            let connection_vec = match connection_queue.read().unwrap().get(&thread_id) {
                Some(v) => v.clone(),
                None => panic!("Error retrieving id {}. Closing Websocket.", thread_id),
            };

            if connection_vec.len() == 0 {
                thread::sleep(Duration::from_millis(10));

                ping_cnt += 1;

                if ping_cnt == 1000 {
                    match sender.send_message(&OwnedMessage::Ping(thread_id.to_string().as_bytes().to_vec())) {
                        Ok(v) => v,
                        Err(e) => { 
                            println!("Error sending message {}. Closing Websocket {}", e, thread_id);
                            
                            subscriber_map.write().unwrap().get_mut(&key).unwrap().remove(&thread_id);
                            connection_queue.write().unwrap().remove(&thread_id);
                            
                            return;
                        },
                    }

                    ping_cnt = 0;
                }
                
                continue;
            }

            match connection_queue.write().unwrap().get_mut(&thread_id) {
                Some(v) => v.clear(),
                None => panic!("Error retrieving id {}. Closing Websocket.", thread_id),
            };

            for update in connection_vec.iter() {
                match sender.send_message(&OwnedMessage::Text(update.to_string())) {
                    Ok(v) => v,
                    Err(e) => { 
                        println!("Error sending message {}. Closing Websocket {}", e, thread_id);

                        subscriber_map.write().unwrap().get_mut(&key).unwrap().remove(&thread_id);
                        connection_queue.write().unwrap().remove(&thread_id);
                        
                        return;
                    },
                }
            }
        }
    });
}

fn read_message(message: OwnedMessage) -> String {
    match message {
        OwnedMessage::Text(txt) => txt.parse().unwrap(),
        _ => panic!("Cannot read message"),
    }
}

pub fn parse_json(json_data: &str) -> (String,String) {
    let mut tmp: String = String::new();
    let mut key: String = String::new();

    for p in json_data.chars() {
        if p == ' ' || p == '\n' || p == '\t' || p == '\"' || p == '{' || p == '}' { continue; }
        
        if p == ':' || p == ',' {
            if key.len() == 0 { key = tmp; } 
            else { break; }
            
            tmp = String::new();

            continue;
        }

        tmp.push(p);
    }

    (key, tmp)
}