use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use chrono::Utc;

use crate::schema;
use crate::schema::rooms::dsl::*;
use crate::schema::rooms::columns::name;
use diesel::prelude::*;
use diesel::result::QueryResult;
use crate::models::Room;
use crate::schema::rooms;

type Tx = mpsc::UnboundedSender<Message>;
type RoomMap = HashMap<String, HashMap<String, Tx>>;
type PeerMap = Arc<Mutex<RoomMap>>;
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;



pub fn load_rooms_from_db(pool: &DbPool, peers: &PeerMap) {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let rooms_result: QueryResult<Vec<Room>> = rooms::table
        .select(Room::as_select())
        .load(&mut conn);

    match rooms_result {
        Ok(loaded_rooms) => {
            let mut peers = peers.lock().unwrap();
            for room in loaded_rooms {
                peers.entry(room.name).or_insert_with(HashMap::new);
            }
        }
        Err(e) => eprintln!("Error loading rooms: {:?}", e),
    }
}

pub async fn handle_connection(peers: PeerMap, stream: TcpStream, pool: DbPool) {
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
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                let result: Result<_, diesel::result::Error> = conn.transaction(|conn| {
                    let existing_room = rooms
                        .filter(name.eq(&room_name))
                        .select(Room::as_select())
                        .first::<Room>(conn)
                        .optional()?;
        
                    if existing_room.is_none() {
                        let new_room = Room {
                            id: None,
                            name: room_name.clone(),
                            created_by: addr.to_string(),
                            created_at: Utc::now().naive_utc(),
                        };
                        diesel::insert_into(rooms)
                            .values(&new_room)
                            .execute(conn)?;
                        peers.entry(room_name.clone()).or_insert_with(HashMap::new);
                        tx.send(Message::Text(format!("\x1b[94mRoom '{}' created.\x1b[0m", room_name))).unwrap();
                        Ok(())
                    } else {
                        tx.send(Message::Text(format!("\x1b[91mRoom '{}' already exists.\x1b[0m", room_name))).unwrap();
                        Ok(())
                    }
                });
                if let Err(e) = result {
                    eprintln!("Database error: {:?}", e);
                }
            }
        } else if message.starts_with("/room") {
            let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            if !room_name.is_empty() {
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                let room_exists: bool = diesel::select(diesel::dsl::exists(
                    rooms::table.filter(rooms::name.eq(&room_name))
                )).get_result(&mut conn).unwrap_or(false);

                if room_exists {
                    if !current_room.is_empty() {
                        if let Some(old_room) = peers.get_mut(&current_room) {
                            old_room.remove(&addr.to_string());
                        }
                    }
                    peers.entry(room_name.clone()).or_insert_with(HashMap::new).insert(addr.to_string(), tx.clone());
                    current_room = room_name.clone();
                    tx.send(Message::Text(format!("\x1b[94mJoined room '{}'.\x1b[0m", room_name))).unwrap();
                } else {
                    tx.send(Message::Text(format!("\x1b[91mRoom '{}' does not exist.\x1b[0m", room_name))).unwrap();
                }
            }
        } else if message == "/leave" {
            if !current_room.is_empty() {
                if let Some(room) = peers.get_mut(&current_room) {
                    room.remove(&addr.to_string());
                    tx.send(Message::Text(format!("\x1b[91mLeft room '{}'.\x1b[0m", current_room))).unwrap();
                    current_room.clear();
                }
            } else {
                tx.send(Message::Text("\x1b[91mYou are not in any room.\x1b[0m".to_string())).unwrap();
            }
        } else if message == "/listrooms" {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            let rooms_result: QueryResult<Vec<String>> = rooms::table
                .select(rooms::name)
                .load::<String>(&mut conn);

            match rooms_result {
                Ok(room_names) => {
                    let rooms_str = room_names.join(", ");
                    tx.send(Message::Text(format!("\x1b[94mRooms: {}\x1b[0m", rooms_str))).unwrap();
                }
                Err(e) => {
                    eprintln!("Error listing rooms: {:?}", e);
                    tx.send(Message::Text("\x1b[91mError listing rooms.\x1b[0m".to_string())).unwrap();
                }
            }
        } 
        else if !current_room.is_empty() {
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
            tx.send(Message::Text("\x1b[91mYou are not in any room. Use /room <room_name> to join a room.\x1b[0m".to_string())).unwrap();
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