mod value_store;
mod websockets;

use crate::websockets::websocket_server::WebSocketServer;

fn main() {
    let websocket_server = WebSocketServer::new("localhost:9002", "localhost:9004");
    websocket_server.start_server();
}
