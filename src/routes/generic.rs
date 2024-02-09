use poem::{get, handler, web::{websocket::{Message, WebSocket}, Data}, Route};

#[handler]
fn index() -> String {
    "Welcome to Liquid Breakout Backend site. Visit /docs for documentation.".to_string()
}

#[handler]
fn websocket(ws: WebSocket, sender: Data<&tokio::sync::broadcast::Sender<String>>) {

}

#[derive(Clone)]
pub struct WebsocketIoQueue {
    client: String,
    action: String,
    bgm: Option<String>,
    start_time: Option<i64>
}

#[derive(Clone)]
pub struct GenericRoutes {
    websocket_io_queue: Vec<WebsocketIoQueue>
}
impl GenericRoutes {
    pub fn new() -> Self {
        Self { websocket_io_queue: Vec::new() }
    }

    pub fn collect(&self) -> Route {
        Route::new()
            .at("/", get(index))
    }
}