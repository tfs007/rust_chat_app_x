use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::schema;
use crate::schema::rooms::dsl::*;
use crate::schema::rooms::columns::name;
use diesel::prelude::*;
use diesel::result::QueryResult;
use crate::models::Room;
use crate::schema::rooms;
use crate::schema::users;

use std::error::Error;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};

use core::net::SocketAddr;

use diesel::result::Error as DieselError;
use chrono::Utc;

use crate::auth;

use tokio_tungstenite::WebSocketStream;

type Tx = mpsc::UnboundedSender<Message>;
type RoomMap = HashMap<String, HashMap<String, Tx>>;
type PeerMap = Arc<Mutex<RoomMap>>;
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// Define User struct
#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub created_at: NaiveDateTime,
}

async fn send_custom_message(stream: &mut WebSocketStream<TcpStream>, message: String) -> Result<(), Box<dyn std::error::Error>> {
    stream.send(Message::Text(message)).await?;
    Ok(())
}


// Function to hash password
fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    let hashed_password = hash(password, DEFAULT_COST)?;
    Ok(hashed_password)
}

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



pub fn register_user(u_name : String, p_hash: String, conn: &mut SqliteConnection) -> Result<(), DieselError> {
    use crate::schema::users;
    use crate::models::User;
    let new_user = User {
        id: None,
        username: u_name,
        password_hash: p_hash,
        created_at: Utc::now().naive_utc(),

    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?;

    Ok(())

}


pub fn login_user(u_name : String, p_hash: String, conn: &mut SqliteConnection)-> String {
    use crate::schema::users;
    use crate::models::User;
    println!("Username: {}; pwd: {}", u_name, p_hash);

    let result = users::table
        .filter(users::username.eq(&u_name))
        .filter(users::password_hash.eq(&p_hash))
        .first::<User>(conn);

    match result {
        Ok(_) => "ok".to_string(),
        Err(diesel::result::Error::NotFound) => "notok".to_string(),
        Err(e) => {
            eprintln!("Database error during login: {:?}", e);
            "notok".to_string()
        }
    }
}

pub fn check_login_user(u_name : String, p_hash: String, conn: &mut SqliteConnection)-> bool {
    use crate::schema::users;
    use crate::models::User;
    println!("Username: {}; pwd: {}", u_name, p_hash);

    let result = users::table
        .filter(users::username.eq(&u_name))
        .filter(users::password_hash.eq(&p_hash))
        .first::<User>(conn);

    match result {
        Ok(_) => true,
        Err(diesel::result::Error::NotFound) => false,
        Err(e) => {
            eprintln!("Database error during login: {:?}", e);
            false
        }
    }
}

// pub fn check_message_login(message : String) -> bool {
//     let mut conn = pool.get().expect("couldn't get db connection from pool");
//     let u_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
//     let h_pwd = message.split_whitespace().nth(2).unwrap_or("").to_string();
//     let login_result = check_login_user(u_name, h_pwd, &mut conn);
//     println!("Login result: {}", login_result);
//     return login_result;
//     // let mut words: Vec<&str> = message.split_whitespace().collect();
//     // return true;
//     // if words.len() <= 2 {
//     //     println!("message too short");
//     //     // return;
//     // } else {
//     //     return check_login_user()
//     // }

// }


pub fn create_user_token(u_name: String, token: String,conn: &mut SqliteConnection )-> Result<(), DieselError> {
    use crate::schema::auth_tokens;
    use crate::models::AuthToken; 
    let new_token = AuthToken {
        id: None, 
        username: u_name,
        auth_token: token,
        created_at: Utc::now().naive_utc(),
    };

    diesel::insert_into(auth_tokens::table)
        .values(&new_token)
        .execute(conn)?;

    Ok(())

}

pub fn create_room_entry(room_name: String, creator: String, conn: &mut SqliteConnection) -> Result<(), DieselError> {
    use crate::schema::rooms;
    use crate::models::Room;

    let new_room = Room {
        id: None,
        name: room_name,
        created_by: creator,
        created_at: Utc::now().naive_utc(),
    };

    diesel::insert_into(rooms::table)
        .values(&new_room)
        .execute(conn)?;

    Ok(())
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
        
         
        if message.starts_with("/createroom") 
        {
            // println!("I AM in CREATEROOM");
            // >>>

            let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            if !room_name.is_empty() 
            {
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                let u_name = message.split_whitespace().nth(2).unwrap_or("").to_string();
                let h_pwd = message.split_whitespace().nth(3).unwrap_or("").to_string();
                let login_result = login_user(u_name, h_pwd, &mut conn);
                if login_result == "ok" {
                // let mut conn = pool.get().expect("couldn't get db connection from pool");
                // >>> ->
                match create_room_entry(room_name.clone(), addr.to_string(), &mut conn) {
                    Ok(_) => {
                        peers.entry(room_name.clone()).or_insert_with(HashMap::new);
                        tx.send(Message::Text(format!("\x1b[94mRoom '{}' created.\x1b[0m", room_name))).unwrap();
                    },
                    Err(e) => {
                        eprintln!("Database error: {:?}", e);
                        tx.send(Message::Text(format!("\x1b[91mFailed to create room '{}'.It already exists.\x1b[0m", room_name))).unwrap();
                    }
                }
            } else {
                tx.send(Message::Text("\x1b[91mPlease log in.\x1b[0m".to_string())).unwrap();
            } 
            }
                

        } else if message.starts_with("/register")
        {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            let u_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            let h_pwd = message.split_whitespace().nth(2).unwrap_or("").to_string();
            // tx.send(Message::Text(format!("\x1b[91m'{}. User name: {}'. hashed pwd: {}\x1b[0m", "Wanna register".to_string(),u_name,h_pwd))).unwrap();

            match register_user(u_name.clone(),h_pwd.clone(),&mut conn) {
                Ok(_) => {
                    tx.send(Message::Text(format!("\x1b[94mUser '{}' created.\x1b[0m", u_name))).unwrap();
                },
                Err(e) => {
                    eprintln!("Database error: {:?}", e);
                    tx.send(Message::Text(format!("\x1b[91mFailed to create user '{}'.It already exists.\x1b[0m", u_name))).unwrap();
                }
            }
        } else if message.starts_with("/login")
        {
            //login function 
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            let u_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            let h_pwd = message.split_whitespace().nth(2).unwrap_or("").to_string();
            let login_result = login_user(u_name, h_pwd, &mut conn);
            println!("Login result: {}", login_result);
            tx.send(Message::Text(login_result.to_string()));
            
        }
        else if message.starts_with("/room") 
        {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            let u_name = message.split_whitespace().nth(2).unwrap_or("").to_string();
            let h_pwd = message.split_whitespace().nth(3).unwrap_or("").to_string();
            let login_result = login_user(u_name, h_pwd, &mut conn);
            if login_result == "ok" {
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

            } else {
                tx.send(Message::Text("\x1b[91mPlease log in.\x1b[0m".to_string())).unwrap();
            }
            // >>
            
            // <<
        } else if message.starts_with("/leave")  {
            if !current_room.is_empty() {
                if let Some(room) = peers.get_mut(&current_room) {
                    room.remove(&addr.to_string());
                    tx.send(Message::Text(format!("\x1b[91mYou left room '{}'.\x1b[0m", current_room))).unwrap();
                    current_room.clear();
                }
            } else {
                tx.send(Message::Text("\x1b[91mYou are not in any room.\x1b[0m".to_string())).unwrap();
            }
        } else if message.starts_with("/listrooms")
        {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            let u_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            let h_pwd = message.split_whitespace().nth(2).unwrap_or("").to_string();
            let login_result = login_user(u_name, h_pwd, &mut conn);
            if login_result == "ok" {
                println!("List room passed");
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
            } else {
                println!("List room failed");
                tx.send(Message::Text("\x1b[91mPlease log in.\x1b[0m".to_string())).unwrap();
            }
            
        } else if message.starts_with("/listusers") {
            let room_name = message.split_whitespace().nth(1).unwrap_or("").to_string();
            if !room_name.is_empty() {
                if let Some(room) = peers.get(&room_name) {
                    let users: Vec<String> = room.keys().cloned().collect();
                    let users_str = users.join(", ");
                    tx.send(Message::Text(format!("\x1b[94mUsers in room '{}': {}\x1b[0m", room_name, users_str))).unwrap();
                } else {
                    tx.send(Message::Text(format!("\x1b[91mRoom '{}' does not exist.\x1b[0m", room_name))).unwrap();
                }
            } else {
                tx.send(Message::Text("\x1b[91mPlease specify a room name.\x1b[0m".to_string())).unwrap();
            }
        }
        else if message == "/register" {
            tx.send(Message::Text("\x1b[91mLet's register...\x1b[0m".to_string())).unwrap();
            // let response = crate::auth::register_user();
            // tx.send(Message::Text(response)).await.unwrap();
        }  else if message == "/logout" {
            tx.send(Message::Text("\x1b[91mLet's logout...\x1b[0m".to_string())).unwrap();
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