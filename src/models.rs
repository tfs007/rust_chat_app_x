use diesel::prelude::*;
use crate::schema::rooms;
use crate::schema::auth_tokens;
use crate::schema::users;
use crate::schema::chat_room_chats;
use crate::schema::direct_msgs;
use crate::schema::live_users;


#[derive(Queryable, Insertable, Selectable)]
pub struct Room {
    pub id: Option<i32>,
    pub name: String,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, Selectable)]
pub struct AuthToken {
    pub id: Option<i32>,
    pub username: String,
    pub auth_token: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, Selectable)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub password_hash: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, Selectable)]
pub struct ChatRoomChat {
    pub id: Option<i32>,
    pub from_username: String,
    pub to_room: String,
    pub message_text: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, Selectable)]
pub struct DirectMsg {
    pub id: Option<i32>,
    pub from_username: String,
    pub to_username: String,
    pub message_text: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, Selectable)]
pub struct LiveUser {
    pub id: Option<i32>,
    pub username: String,
    pub socket_ip: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}




