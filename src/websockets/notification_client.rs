use std::{
    thread,
    time::Duration,
    collections::HashSet,
};

use tungstenite::{
    connect,
    Message,
};

use crate::websockets::ConnectionService;

pub struct NotificationClient {
    ip_client: String,
    connection_service: ConnectionService,
}

impl NotificationClient {
    pub fn new(ip_client: String, connection_service: ConnectionService) -> Self {
        NotificationClient {
            ip_client: ip_client, 
            connection_service: connection_service,
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
                        let ohlc_model = self.connection_service.add_ohlc_json(text);
                        let ids_to_update:HashSet<usize> = self.connection_service.get_subscribers(&ohlc_model.stock_name);

                        self.connection_service.add_events(ids_to_update, ohlc_model.to_string());
                    }
                    _ => (),
                }
            }
        }
    }
}