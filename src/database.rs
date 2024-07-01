// use diesel::prelude::*;
// use diesel::r2d2::{ConnectionManager, PooledConnection};
// use std::sync::{Arc, Mutex};
// use crate::models::Room;
// use crate::schema::rooms;
// use crate::PeerMap;

// type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// pub fn load_rooms_from_db(pool: &DbPool, peers: &PeerMap) {
//     let mut conn = pool.get().expect("couldn't get db connection from pool");
//     let rooms: Vec<Room> = rooms::table
//         .load(&mut conn)
//         .expect("Error loading rooms");

//     let mut peers = peers.lock().unwrap();
//     for room in rooms {
//         peers.entry(room.name.clone()).or_insert_with(HashMap::new);
//     }
// }

// pub async fn create_room(room_name: &str, creator_addr: &str, peers: &mut PeerMap, pool: &DbPool) -> Result<(), diesel::result::Error> {
//     let mut conn = pool.get().expect("couldn't get db connection from pool");
//     let result: Result<_, diesel::result::Error> = conn.transaction(|conn| {
//         use crate::schema::rooms::dsl::*;
//         let existing_room = rooms
//             .filter(name.eq(room_name))
//             .first::<Room>(conn)
//             .optional()?;

//         if existing_room.is_none() {
//             let new_room = Room {
//                 id: 0, // This will be auto-incremented by SQLite
//                 name: room_name.to_string(),
//                 created_by: creator_addr.to_string(),
//                 created_at: chrono::Utc::now().naive_utc(),
//             };

//             diesel::insert_into(rooms)
//                 .values(&new_room)
//                 .execute(conn)?;

//             peers.entry(room_name.to_string()).or_insert_with(HashMap::new);
//             Ok(())
//         } else {
//             Err(diesel::result::Error::RollbackTransaction)
//         }
//     });

//     result
// }
