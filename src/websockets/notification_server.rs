use std::{
    thread,
    time::Duration,
    collections::HashMap,
    net::{TcpStream, TcpListener},
};

use tungstenite::{
    accept,
    protocol::{Role, WebSocket},
    Message,
};

use crate::websockets::ConnectionService;

pub struct NotificationServer {
    ip_server: String,
    connection_service: ConnectionService,
}

impl NotificationServer {
    pub fn new(ip_server: String, connection_service: ConnectionService) -> Self {
        NotificationServer{ 
            ip_server: ip_server,
            connection_service: connection_service,
        }
    }

    pub fn start_server(&self) {
        let server = TcpListener::bind(self.ip_server.clone()).unwrap();
        let connection_service = self.connection_service.clone();

        thread::spawn(move || {
            for stream in server.incoming() {
                let connection_service_clone = connection_service.clone();

                thread::spawn(move || {
                    let stream_read = stream.unwrap();
                    let send_stream = stream_read.try_clone().unwrap();

                    let websocket_read = accept(stream_read).unwrap();
                    let websocket_send = WebSocket::from_raw_socket(send_stream, Role::Server, None);

                    let id = connection_service_clone.add_subscriber();
                    
                    start_websocket_receiver(
                        websocket_read, 
                        connection_service_clone.clone(),
                        id
                    );
                    
                    start_websocket_sender(
                        websocket_send, 
                        connection_service_clone, 
                        id
                    );
        
                    println!("Spawned websocket {}", id);
                });
            }
        });
    }
}

fn start_websocket_receiver(mut receiver: WebSocket<TcpStream>,
                            connection_service:ConnectionService,
                            id: usize) {
    thread::spawn(move || {
        let mut key_stock:String = String::new();

        loop {
            let message_json:String = match receiver.read() {
                Ok(message) => match message {
                    msg @ Message::Text(_) => msg.into_text().unwrap(),
                    _msg @ Message::Ping(_) | _msg @ Message::Pong(_) => continue,
                    _ => break,
                },
                Err(e) =>{
                    println!("Error in message {} thread: {}", e, id);
                    break;
                },
            };

            let parsed_json = parse_json(&message_json);

            let stock_name = match parsed_json.get("stock") {
                Some(v) => v.to_string(),
                None => {
                    println!("Error with stock in thread {}", id);
                    continue;
                }
            };
            
            connection_service.remove_stock_subscription(id, &key_stock);
            key_stock = stock_name;
            connection_service.add_stock_subscription(id, &key_stock);
        }

        println!("Closing Receiver thread {}", id);
        connection_service.remove_stock_subscription(id, &key_stock);
    });
}

fn start_websocket_sender(mut sender: WebSocket<TcpStream>,
                          connection_service: ConnectionService,
                          id: usize) {
    thread::spawn(move || {
        let mut ping_cnt:usize = 0;

        loop {
            let connection_vec = connection_service.read_events(&id);

            if connection_vec.len() == 0 {
                thread::sleep(Duration::from_millis(10));

                if !send_ping(&mut sender, &mut ping_cnt) { 
                    break; 
                }
                
                continue;
            }

            for update in connection_vec.into_iter() {
                match sender.send(Message::Text(update)) {
                    Ok(v) => v,
                    Err(_) => break,
                }
            }
        }

        println!("Error sending message. Closing Websocket {}", id);
        connection_service.remove_subscriber(id);
    });

}

fn send_ping(sender: &mut WebSocket<TcpStream>, ping_cnt: &mut usize) -> bool {
    *ping_cnt += 1;

    if *ping_cnt < 100 { 
        return true; 
    }

    match sender.send(Message::Ping(Vec::new())) {
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
        match p {
            ' ' | '\n' | '\t' | '\"' | '{' | '}' => (),
            ':' | ',' => {
                match key.len() {
                    0 => key = tmp,
                    _ => {
                        parsed_json.insert(key, tmp);
                        key = String::new();
                    }
                };
                
                tmp = String::new();
            },
            _ => tmp.push(p),
        };
    }

    if key.len() > 0 && tmp.len() > 0 { parsed_json.insert(key, tmp); } 

    parsed_json
}