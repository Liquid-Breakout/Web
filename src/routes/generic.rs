use std::sync::{Arc, Mutex};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use poem::{get, handler, web::{websocket::{Message, WebSocket}, Data, Path}, EndpointExt, IntoResponse, Route};

#[handler]
fn index() -> String {
    "Welcome to Liquid Breakout Backend site. Visit /docs for documentation.".to_string()
}

// Used for sending data to IO clients
#[derive(Clone, Serialize)]
pub struct WebsocketIoStruct {
    pub client: String,
    pub action: String,
    pub bgm: Option<String>,
    pub start_time: Option<u64>
}

#[handler]
fn websocket(
    Path((join_type, username)): Path<(String, String)>,
    ws: WebSocket,
    websocket_io_queue: Data<&Arc<Mutex<Vec<WebsocketIoStruct>>>>,
    sender: Data<&tokio::sync::broadcast::Sender<String>>
) -> impl IntoResponse {
    let websocket_io_queue = websocket_io_queue.clone();
    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    ws.on_upgrade(move |socket| async move {
        let (mut sink, _) = socket.split();

        // This checks for data
        if join_type == "io" {
            tokio::spawn(async move {
                loop {
                    let mut queue: Vec<WebsocketIoStruct> = Vec::new();
                    {
                        let mut io_queue_mutex = websocket_io_queue.lock().unwrap();
                        (*io_queue_mutex)
                            .iter()
                            .position(|e| e.client != username)
                            .map(|e| {
                                queue.push((*io_queue_mutex).get(e).unwrap().to_owned());
                                (*io_queue_mutex).remove(e);
                            });
                    }

                    let last = queue.last();
                    if let Some(data) = last {
                        let data = serde_json::to_string(data).unwrap();
                        if sender.send(data).is_err() {
                            break;
                        }
                    }
                }
            });

            // This is basically sending the data to all clients connected (in this case, "sinks")
            tokio::spawn(async move {
                while let Ok(msg) = receiver.recv().await {
                    if sink.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            });
        }
    })
}

pub struct GenericRoutes {
    pub websocket_io_queue: Arc<Mutex<Vec<WebsocketIoStruct>>>
}
impl GenericRoutes {
    pub fn new() -> Self {
        Self { websocket_io_queue: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn collect(&self) -> Route {
        Route::new()
            .at("/", get(index))
            .at("/websocket/:join_type/:username", websocket.data(self.websocket_io_queue.clone()).data(tokio::sync::broadcast::channel::<String>(1).0))
    }
}