// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
// use tokio::sync::mpsc;
// use tokio_tungstenite::tungstenite::Message;
// use diesel::r2d2::PooledConnection;
// use diesel::SqliteConnection;
// use crate::models::Room;
// use crate::schema::rooms;
// use crate::database;

// type Tx = mpsc::UnboundedSender;
// type RoomMap = HashMap<String, Tx>;
// type PeerMap = Arc<Mutex<HashMap<String, RoomMap>>>;
// type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

// pub async fn create_room(message: &str, creator_addr: &str, peers: &mut PeerMap, pool: &DbPool) {
//     let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
//     if !room_name.is_empty() {
//         if let Err(e) = database::create_room(&room_name, creator_addr, peers, pool).await {
//             eprintln!("Error creating room: {:?}", e);
//         } else {
//             println!("Room '{}' created.", room_name);
//         }
//     } else {
//         let mut peers = peers.lock().unwrap();
//         peers.entry("".to_string()).or_insert_with(HashMap::new)
//             .get_mut(creator_addr)
//             .unwrap()
//             .send(Message::Text("\x1b[91mInvalid room name.\x1b[0m".to_string()))
//             .unwrap();
//     }
// }

// pub async fn join_room(message: &str, addr: &str, peers: &mut PeerMap, current_room: &mut String, tx: &Tx) {
//     let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
//     if !room_name.is_empty() {
//         let mut peers = peers.lock().unwrap();
//         if let Some(old_room) = peers.get_mut(current_room) {
//             old_room.remove(&addr.to_string());
//         }
//         peers.entry(room_name.clone()).or_insert_with(HashMap::new)
//             .insert(addr.to_string(), tx.clone());
//         *current_room = room_name.clone();
//         tx.send(Message::Text(format!("\x1b[94mJoined room '{}'.\x1b[0m", room_name))).unwrap();
//     } else {
//         tx.send(Message::Text("\x1b[91mInvalid room name.\x1b[0m".to_string())).unwrap();
//     }
// }

// pub async fn leave_room(addr: &str, peers: &mut PeerMap, current_room: &mut String, tx: &Tx) {
//     let mut peers = peers.lock().unwrap();
//     if let Some(room) = peers.get_mut(current_room) {
//         room.remove(&addr.to_string());
//         tx.send(Message::Text(format!("\x1b[91mLeft room '{}'.\x1b[0m", current_room))).unwrap();
//         current_room.clear();
//     } else {
//         tx.send(Message::Text("\x1b[91mYou are not in any room.\x1b[0m".to_string())).unwrap();
//     }
// }

// pub async fn list_rooms(pool: &DbPool, tx: &Tx) {
//     let mut conn = pool.get().expect("couldn't get db connection from pool");
//     let rooms: Vec<String> = rooms::table
//         .select(rooms::name)
//         .load(&mut conn)
//         .expect("Error loading rooms");
//     let rooms_str = rooms.join(", ");
//     tx.send(Message::Text(format!("\x1b[94mRooms: {}\x1b[0m", rooms_str))).unwrap();
// }

// pub async fn broadcast_message(message: &str, sender_addr: &str, peers: &PeerMap, current_room: &str) {
//     let peers = peers.lock().unwrap();
//     if let Some(room) = peers.get(current_room) {
//         let broadcast_recipients: Vec<_> = room
//             .iter()
//             .filter(|(peer_addr, _)| *peer_addr != sender_addr.to_string())
//             .map(|(_, ws_sink)| ws_sink.clone())
//             .collect();
//         for recp in broadcast_recipients {
//             recp.send(Message::Text(message.to_string())).unwrap();
//         }
//     }
// }

// pub fn remove_client_from_room(addr: &str, peers: &mut PeerMap, current_room: &str) {
//     let mut peers = peers.lock().unwrap();
//     if let Some(room) = peers.get_mut(current_room) {
//         room.remove(&addr.to_string());
//     }
// }
