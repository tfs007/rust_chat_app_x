use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

type Tx = mpsc::UnboundedSender<Message>;
type RoomMap = HashMap<String, HashMap<String, Tx>>;
type PeerMap = Arc<Mutex<RoomMap>>;

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
    let (outgoing, incoming) = ws_stream.split();
    
    let mut current_room = String::new();
    
    let broadcast_incoming = incoming.try_for_each(|msg| {
        let message = msg.to_text().unwrap();
        println!("Received a message from {}: {}", addr, message);
        
        let mut peers = peers.lock().unwrap();
        
        if message.starts_with("/createroom") {
            let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            if !room_name.is_empty() {
                peers.entry(room_name.clone()).or_insert_with(HashMap::new);
                tx.send(Message::Text(format!("Room '{}' created.", room_name))).unwrap();
            }
        } else if message.starts_with("/room") {
            let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            if !room_name.is_empty() {
                if peers.contains_key(&room_name) {
                    // First, remove from the current room if any
                    if !current_room.is_empty() {
                        if let Some(old_room) = peers.get_mut(&current_room) {
                            old_room.remove(&addr.to_string());
                        }
                    }
                    // Then, join the new room
                    peers.get_mut(&room_name).unwrap().insert(addr.to_string(), tx.clone());
                    current_room = room_name.clone();
                    tx.send(Message::Text(format!("Joined room '{}'.", room_name))).unwrap();
                } else {
                    tx.send(Message::Text(format!("Room '{}' does not exist.", room_name))).unwrap();
                }
            }
        } else if message == "/leave" {
            if !current_room.is_empty() {
                if let Some(room) = peers.get_mut(&current_room) {
                    room.remove(&addr.to_string());
                }
                tx.send(Message::Text(format!("Left room '{}'.", current_room))).unwrap();
                current_room.clear();
            } else {
                tx.send(Message::Text("You are not in any room.".to_string())).unwrap();
            }
        } else if message == "/listrooms" {
            let rooms: Vec<_> = peers.keys().cloned().collect();
            let rooms_str = rooms.join(", ");
            tx.send(Message::Text(format!("/rooms:{}", rooms_str))).unwrap();
        } else if !current_room.is_empty() {
            if let Some(room) = peers.get(&current_room) {
                let broadcast_recipients: Vec<_> = room
                    .iter()
                    .filter(|(peer_addr, _)| **peer_addr != addr.to_string())
                    .map(|(_, ws_sink)| ws_sink.clone())
                    .collect();
                
                for recp in broadcast_recipients {
                    recp.send(msg.clone()).unwrap();
                }
            }
        } else {
            tx.send(Message::Text("You are not in any room. Use /room <room_name> to join a room.".to_string())).unwrap();
        }
        
        futures_util::future::ok(())
    });
    
    let receive_from_others = UnboundedReceiverStream::new(rx).map(Ok).forward(outgoing);
    
    futures_util::pin_mut!(broadcast_incoming, receive_from_others);
    futures_util::future::select(broadcast_incoming, receive_from_others).await;
    
    println!("{} disconnected", &addr);
    
    let mut peers = peers.lock().unwrap();
    if !current_room.is_empty() {
        if let Some(room) = peers.get_mut(&current_room) {
            room.remove(&addr.to_string());
        }
    }
}