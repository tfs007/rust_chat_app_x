use diesel::prelude::*;
use crate::schema::rooms;

#[derive(Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::schema::rooms)]
pub struct Room {
    pub id: Option<i32>,
    pub name: String,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}