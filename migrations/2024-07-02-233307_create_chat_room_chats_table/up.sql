-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE chat_room_chats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_username TEXT NOT NULL ,
    to_room TEXT NOT NULL,
    message_text TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);