use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::mpsc;
use diesel::sqlite::SqliteConnection;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;


mod schema;
mod connection_handler;
mod models;
mod auth;

use models::Room;

type Tx = mpsc::UnboundedSender<Message>;
type RoomMap = HashMap<String, HashMap<String, Tx>>;
type PeerMap = Arc<Mutex<RoomMap>>;
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

use diesel::query_dsl::InternalJoinDsl;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);
    
    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));
    
    
    connection_handler::load_rooms_from_db(&pool, &peers);


    while let Ok((stream, _)) = listener.accept().await {
        let peer_map = peers.clone();
        let db_pool = pool.clone();
        tokio::spawn(connection_handler::handle_connection(peer_map, stream, db_pool));
    }
}

