use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::Deserialize;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

// The shared map of connected SSE clients: client_id -> message sender
// Arc<Mutex<...>> lets it be safely shared between the worker and the controller
pub type ClientMap = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>;

// Query param the frontend sends: /notifications/stream?client_id=abc123
#[derive(Deserialize)]
pub struct StreamQuery {
    pub client_id: String,
}

pub async fn stream(
    Query(query): Query<StreamQuery>,
    State(clients): State<ClientMap>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    // Create a channel just for this client connection
    let (tx, rx) = mpsc::channel::<String>(32);

    // Register this client so the worker can find it
    clients.lock().await.insert(query.client_id.clone(), tx);

    // Clean up when client disconnects
    let client_id = query.client_id.clone();
    let clients_cleanup = clients.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(0)).await;
        // The mpsc channel will close naturally on disconnect,
        // but we also remove the dead entry from the map
        clients_cleanup.lock().await.remove(&client_id);
    });

    // Stream messages as SSE events
    let stream = ReceiverStream::new(rx)
        .map(|msg| Ok(Event::default().data(msg)));

    Sse::new(stream).keep_alive(KeepAlive::default())
}