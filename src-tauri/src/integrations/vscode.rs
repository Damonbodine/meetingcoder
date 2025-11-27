use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub struct VSCodeServer {
    app_handle: AppHandle,
    clients: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>,
}

impl VSCodeServer {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self) {
        let clients = self.clients.clone();
        let app_handle = self.app_handle.clone();

        tauri::async_runtime::spawn(async move {
            let addr = "127.0.0.1:3000";
            let listener = TcpListener::bind(&addr).await.expect("Failed to bind VSCode server");
            log::info!("VSCode WebSocket server listening on: {}", addr);

            while let Ok((stream, _)) = listener.accept().await {
                let clients = clients.clone();
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, clients, app_handle).await {
                        log::error!("Error handling VSCode connection: {}", e);
                    }
                });
            }
        });
    }

    pub fn broadcast(&self, message: String) {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|client| {
            client.send(Message::Text(message.clone())).is_ok()
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    clients: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>,
    _app_handle: AppHandle,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    log::info!("New VSCode connection established");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    clients.lock().unwrap().push(tx);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if msg.is_text() {
            let text = msg.to_text()?;
            log::info!("Received from VSCode: {}", text);
            // TODO: Parse context update and store it
        }
    }

    Ok(())
}
