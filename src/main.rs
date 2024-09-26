mod value_store;
mod websockets;

use crate::websockets::websocket_server::WebSocketServer;

fn main() {
    let websocket_server = WebSocketServer::new("localhost:9002", "152.53.36.150:9004");
    websocket_server.start_server();
}
