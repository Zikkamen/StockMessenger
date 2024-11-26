use crate::websockets::{NotificationClient, NotificationServer, ConnectionService};

pub struct WebSocketServer {
    ip_server: String,
    ip_client: String,
}

impl WebSocketServer {
    pub fn new(ip_server: &str, ip_client: &str) -> Self {
        WebSocketServer { 
            ip_server: ip_server.to_owned(), 
            ip_client: ip_client.to_owned() 
        }
    }

    pub fn start_server(&self) {
        let connection_service = ConnectionService::new();

        let notification_server = NotificationServer::new(
            self.ip_server.clone(),
            connection_service.clone(),
        );
        
        notification_server.start_server();

        let mut notification_client = NotificationClient::new(
            self.ip_client.clone(),
            connection_service,
        );

        notification_client.start_client();
    }
}