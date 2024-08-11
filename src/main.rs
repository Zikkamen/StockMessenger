use std::thread;
use std::sync::RwLock;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use websocket::sync::Server;
use websocket::{OwnedMessage, Message, ClientBuilder};
use websocket::server::upgrade::WsUpgrade;

mod value_store;

use crate::value_store::stock_information_cache::StockInformationCache;

fn main() {
    let connection_queue = Arc::new(RwLock::new(HashMap::<usize, Vec<String>>::new()));
    let stock_information_cache = Arc::new(RwLock::new(StockInformationCache::new()));

    start_websocketserver(Arc::clone(&connection_queue), Arc::clone(&stock_information_cache));
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

                    stock_information_cache.write().unwrap().add_json(&text); 

                    let iter_keys:Vec<usize> = connection_queue.read().unwrap().keys().copied().collect();
                    let mut connection_vec = connection_queue.write().unwrap();

                    for key in iter_keys.iter() {
                        match connection_vec.get_mut(key) {
                            Some(v) => v.push(text.clone()),
                            None => continue,
                        };
                    }
                }
                _ => (),
            }
        }
    }
}

fn start_websocketserver(connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>, stock_information_cache: Arc<RwLock<StockInformationCache>>){
    let server = Server::bind("127.0.0.1:9002").unwrap();

    thread::spawn(move || {
        let mut id:usize = 0;

        for connection in server.filter_map(Result::ok) {
            connection_queue.write().unwrap().insert(id, stock_information_cache.read().unwrap().get_vec_of_cache());

            start_websocket(connection, Arc::clone(&connection_queue), id);

            println!("Spawned websocket {}", id);
            id += 1;
        }
    });
}

fn start_websocket(connection: WsUpgrade<std::net::TcpStream, Option<websocket::server::upgrade::sync::Buffer>>,
    connection_queue: Arc<RwLock<HashMap::<usize, Vec<String>>>>,
    id: usize) {
    let client = connection.accept().unwrap();
    let (_receiver, mut sender) = client.split().unwrap();

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
                println!("This is the update {} {}", thread_id, update.clone());

                match sender.send_message(&OwnedMessage::Text(update.to_string())) {
                    Ok(v) => v,
                    Err(e) => { 
                        println!("Error sending message {}. Closing Websocket {}", e, thread_id); 
                        
                        connection_queue.write().unwrap().remove(&thread_id);
                        
                        return;
                    },
                }
            }
        }
    });
}
