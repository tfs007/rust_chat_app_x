// use bcrypt::{hash, verify, DEFAULT_COST};
// use diesel::prelude::*;
// use diesel::r2d2::{self, ConnectionManager};
// use dotenv::dotenv;
// use futures_util::{SinkExt, StreamExt};
// use serde::{Deserialize, Serialize};
// use std::env;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tokio_tungstenite::tungstenite::Message;
// use chrono::{Utc, NaiveDateTime};
// use std::error::Error;

// // mod models;
// // mod schema;

// type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// #[derive(Serialize, Deserialize)]
// struct AuthRequest {
//     action: String,
//     username: String,
//     password: String,
// }

// #[derive(Serialize, Deserialize)]
// struct AuthResponse {
//     success: bool,
//     message: String,
// }

// fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
//     let hashed_password = hash(password, DEFAULT_COST)?;
//     Ok(hashed_password)
// }

// pub async fn register_user(username: String, password: String, pool: &Pool<ConnectionManager<SqliteConnection>>) -> Result<String, Box<dyn Error>> {
//     // Hash the password
//     let hashed_password = hash_password(&password)?;

//     // Get a database connection from the pool
//     let conn = pool.get()?;

// }

