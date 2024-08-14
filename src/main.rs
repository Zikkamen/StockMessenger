use std::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

mod value_store;
mod websockets;

use crate::websockets::notification_server::NotificationServer;
use crate::websockets::notification_client::NotificationClient;
use crate::value_store::stock_information_cache::StockInformationCache;

fn main() {
    let connection_queue = Arc::new(RwLock::new(HashMap::<usize, Vec<String>>::new()));
    let stock_information_cache = Arc::new(RwLock::new(StockInformationCache::new()));
    let subscriber_map = Arc::new(RwLock::new(HashMap::<(String, String), Vec<usize>>::new()));

    let notification_server = NotificationServer::new(Arc::clone(&connection_queue), Arc::clone(&subscriber_map), Arc::clone(&stock_information_cache));
    notification_server.start_server();

    let mut notification_client = NotificationClient::new(Arc::clone(&connection_queue), Arc::clone(&subscriber_map), Arc::clone(&stock_information_cache));
    notification_client.start_client();
}
