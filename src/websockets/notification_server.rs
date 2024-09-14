use std::thread;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::collections::HashSet;

use websocket::sync::Server;
use websocket::{OwnedMessage};
use websocket::sync::Writer;
use websocket::sync::Reader;
use websocket::stream::sync::TcpStream;

use crate::value_store::stock_information_cache::StockInformationCache;

pub struct NotificationServer {
    connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    subscriber_map: Arc<RwLock<HashMap::<(String, usize), HashSet<usize>>>>,
    stock_information_cache: Arc<RwLock<StockInformationCache>>,
}

impl NotificationServer {
    pub fn new(connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
               subscriber_map: Arc<RwLock<HashMap::<(String, usize), HashSet<usize>>>>,
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
                         subscriber_map: Arc<RwLock<HashMap::<(String, usize), HashSet<usize>>>>,
                         stock_information_cache: Arc<RwLock<StockInformationCache>>){
    let server = Server::bind("127.0.0.1:9002").unwrap();

    thread::spawn(move || {
        let mut id:usize = 0;

        for connection in server.filter_map(Result::ok) {
            let client = match connection.accept() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let (receiver, sender) = client.split().unwrap();

            connection_queue.write().unwrap().insert(id, Vec::new());
            
            start_websocket_receiver(receiver, Arc::clone(&connection_queue), Arc::clone(&subscriber_map), Arc::clone(&stock_information_cache), id);
            start_websocket_sender(sender, Arc::clone(&connection_queue), id);

            println!("Spawned websocket {}", id);
            id += 1;
        }
    });
}

fn start_websocket_receiver(mut receiver: Reader<TcpStream>,
                            connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
                            subscriber_map: Arc<RwLock<HashMap::<(String, usize), HashSet<usize>>>>,
                            stock_information_cache: Arc<RwLock<StockInformationCache>>,
                            id: usize) {
    thread::spawn(move || {
        let mut key_stock:(String, usize) = (String::new(), 0);

        loop {
            let message_json:String = match receiver.recv_message() {
                Ok(message) => match message {
                    OwnedMessage::Text(txt) => txt.parse().unwrap(),
                    OwnedMessage::Ping(_) => continue,
                    OwnedMessage::Pong(_) => continue,
                    _ => break,
                },
                Err(e) =>{
                    println!("Error in message {} thread: {}", e, id);
                    break;
                },
            };

            let parsed_json:HashMap<String,String> = parse_json(&message_json);

            if !parsed_json.contains_key("stock") || !parsed_json.contains_key("interval") {
                println!("Error with stock and interval in thread {}", id);

                continue;
            }

            match subscriber_map.write().unwrap().get_mut(&key_stock) {
                Some(v) => { v.remove(&id); },
                None => println!("Couldn't find key {:?}", &key_stock),
            };

            let stock_name:String = parsed_json.get("stock").unwrap().to_string();
            let stock_interval = match parsed_json.get("interval").unwrap().parse::<usize>(){ Ok(v) => v, _ => 0 };

            key_stock = (stock_name, stock_interval);

            if &key_stock.0[..] != "*" && !stock_information_cache.read().unwrap().has_key(&key_stock) {
                println!("Couldn't find key {:?}", key_stock);

                continue;
            }

            let key_is_there = subscriber_map.read().unwrap().contains_key(&key_stock);

            match key_is_there {
                true => { subscriber_map.write().unwrap().get_mut(&key_stock).unwrap().insert(id); },
                false => { subscriber_map.write().unwrap().insert(key_stock.clone(), HashSet::from([id])); },
            };
            
            match &key_stock.0[..] {
                "*" => connection_queue.write().unwrap().insert(id, stock_information_cache.read().unwrap().get_vec_dashboard()),
                _ => connection_queue.write().unwrap().insert(id, stock_information_cache.read().unwrap().get_vec_of_stock(&key_stock)),
            };
        }

        println!("Closing Receiver thread {}", id);

        match subscriber_map.write().unwrap().get_mut(&key_stock) {
            Some(v) => { v.remove(&id); },
            None => (),
        };
    });
}

fn start_websocket_sender(mut sender: Writer<TcpStream>,
                   connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
                   id: usize) {
    thread::spawn(move || {
        let mut ping_cnt:usize = 0;

        loop {
            if connection_queue.read().unwrap().len() > 1000 { continue; }

            let connection_vec = match connection_queue.read().unwrap().get(&id) {
                Some(v) => v.clone(),
                None => break,
            };

            if connection_vec.len() == 0 {
                thread::sleep(Duration::from_millis(10));

                if !send_ping(&mut sender, &mut ping_cnt) { break; }
                
                continue;
            }

            match connection_queue.write().unwrap().get_mut(&id) {
                Some(v) => v.clear(),
                None => break,
            };

            for update in connection_vec.iter() {
                match sender.send_message(&OwnedMessage::Text(update.to_string())) {
                    Ok(v) => v,
                    Err(_) => break,
                }
            }
        }

        println!("Error sending message. Closing Websocket {}", id);
        connection_queue.write().unwrap().remove(&id);
    });

}

fn send_ping(sender: &mut Writer<TcpStream>, ping_cnt: &mut usize) -> bool {
    *ping_cnt += 1;

    if *ping_cnt < 100 { return true; }

    match sender.send_message(&OwnedMessage::Ping(Vec::new())) {
        Ok(v) => v,
        Err(_) =>  return false,
    };

    *ping_cnt = 0;

    true
}

pub fn parse_json(json_data: &str) -> HashMap<String ,String> {
    let mut tmp: String = String::new();
    let mut key: String = String::new();

    let mut parsed_json:HashMap<String,String> = HashMap::new();

    for p in json_data.chars() {
        if p == ' ' || p == '\n' || p == '\t' || p == '\"' || p == '{' || p == '}' { continue; }
        
        if p == ':' || p == ',' {
            match key.len() {
                0 => key = tmp,
                _ => {
                    parsed_json.insert(key, tmp);
                    key = String::new();
                }
            };
            
            tmp = String::new();

            continue;
        }

        tmp.push(p);
    }

    if key.len() > 0 && tmp.len() > 0 { parsed_json.insert(key, tmp); } 

    parsed_json
}