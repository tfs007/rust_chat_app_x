use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

type Tx = mpsc::UnboundedSender<Message>;
type RoomMap = HashMap<String, Tx>;
type PeerMap = Arc<Mutex<HashMap<String, RoomMap>>>;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);

    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(peers.clone(), stream));
    }
}

async fn handle_connection(peers: PeerMap, stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (tx, rx) = mpsc::unbounded_channel();
    peers.lock().unwrap().entry(addr.to_string()).or_insert_with(HashMap::new).insert(addr.to_string(), tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!("Received a message from {}: {}", addr, msg.to_text().unwrap());
        let peers = peers.lock().unwrap();

        let broadcast_recipients = peers
            .get(&addr.to_string())
            .unwrap()
            .iter()
            .filter(|(peer_addr, _)| **peer_addr != addr.to_string())
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.send(msg.clone()).unwrap();
        }

        futures_util::future::ok(())
    });

    let receive_from_others = UnboundedReceiverStream::new(rx).map(Ok).forward(outgoing);

    futures_util::pin_mut!(broadcast_incoming, receive_from_others);
    futures_util::future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peers.lock().unwrap().get_mut(&addr.to_string()).unwrap().remove(&addr.to_string());
}