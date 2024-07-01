// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};

// use tokio::net::TcpStream;
// use tokio_tungstenite::tungstenite::Message;
// use futures_util::{SinkExt, StreamExt, TryStreamExt};
// use tokio::sync::mpsc;
// use tokio_stream::wrappers::UnboundedReceiverStream;
// use diesel::r2d2::PooledConnection;
// use diesel::r2d2::ConnectionManager;
// use diesel::SqliteConnection;

// use crate::room_manager;
// use crate::database;
// use crate::models::Room;

// type Tx = mpsc::UnboundedSender;
// type RoomMap = HashMap<String, Tx>;
// type PeerMap = Arc<Mutex<HashMap<String, RoomMap>>>;
// type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// pub async fn handle_connection(peers: PeerMap, stream: TcpStream, pool: DbPool) {
//     let addr = stream.peer_addr().expect("connected streams should have a peer address");
//     println!("Peer address: {}", addr);

//     let ws_stream = tokio_tungstenite::accept_async(stream)
//         .await
//         .expect("Error during the WebSocket handshake occurred");

//     println!("New WebSocket connection: {}", addr);

//     let (tx, rx) = mpsc::unbounded_channel();
//     let (outgoing, incoming) = ws_stream.split();

//     let mut current_room = String::new();

//     let broadcast_incoming = incoming.try_for_each(|msg| {
//         let message = msg.to_text().unwrap();
//         println!("Received a message from {}: {}", addr, message);

//         let mut peers = peers.lock().unwrap();

//         if message.starts_with("/createroom") {
//             room_manager::create_room(&message, &addr, &mut peers, &pool).await;
//         } else if message.starts_with("/room") {
//             room_manager::join_room(&message, &addr, &mut peers, &mut current_room, &tx).await;
//         } else if message == "/leave" {
//             room_manager::leave_room(&addr, &mut peers, &mut current_room, &tx).await;
//         } else if message == "/listrooms" {
//             room_manager::list_rooms(&pool, &tx).await;
//         } else if !current_room.is_empty() {
//             room_manager::broadcast_message(&message, &addr, &peers, &current_room).await;
//         } else {
//             tx.send(Message::Text("\x1b[91mYou are not in any room. Use /room to join a room.\x1b[0m".to_string())).unwrap();
//         }

//         futures_util::future::ok(())
//     });

//     let receive_from_others = UnboundedReceiverStream::new(rx).map(Ok).forward(outgoing);

//     futures_util::pin_mut!(broadcast_incoming, receive_from_others);
//     futures_util::future::select(broadcast_incoming, receive_from_others).await;

//     println!("{} disconnected", &addr);

//     let mut peers = peers.lock().unwrap();
//     if !current_room.is_empty() {
//         room_manager::remove_client_from_room(&addr, &mut peers, &current_room);
//     }
// }
