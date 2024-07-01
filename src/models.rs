use diesel::prelude::*;
use crate::schema::rooms;
use crate::schema::auth_tokens;
use crate::schema::users;

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