use std::sync::{Arc, RwLock};
use std::collections::{HashSet, HashMap};

use crate::value_store::stock_information_cache::StockInformationCache;
use crate::websockets::notification_server::NotificationServer;
use crate::websockets::notification_client::NotificationClient;

pub struct WebSocketServer {
    ip_server: String,
    ip_client: String,
}

impl WebSocketServer {
    pub fn new(ip_server: &str, ip_client: &str) -> Self {
        WebSocketServer { 
            ip_server: ip_server.to_string(), 
            ip_client: ip_client.to_string() 
        }
    }

    pub fn start_server(&self) {
        let connection_queue = Arc::new(RwLock::new(HashMap::<usize, Vec<String>>::new()));
        let stock_information_cache = Arc::new(RwLock::new(StockInformationCache::new()));
        let subscriber_map = Arc::new(RwLock::new(HashMap::<(String, usize), HashSet<usize>>::new()));

        let notification_server = NotificationServer::new(
            self.ip_server.clone(),
            Arc::clone(&connection_queue), 
            Arc::clone(&subscriber_map), 
            Arc::clone(&stock_information_cache)
        );
        
        notification_server.start_server();

        let mut notification_client = NotificationClient::new(
            self.ip_server.clone(),
            Arc::clone(&connection_queue), 
            Arc::clone(&subscriber_map), 
            Arc::clone(&stock_information_cache)
        );

        notification_client.start_client();
    }
}