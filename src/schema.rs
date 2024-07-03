// @generated automatically by Diesel CLI.

diesel::table! {
    auth_tokens (id) {
        id -> Nullable<Integer>,
        username -> Text,
        auth_token -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    chat_room_chats (id) {
        id -> Nullable<Integer>,
        from_username -> Text,
        to_room -> Text,
        message_text -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    direct_msgs (id) {
        id -> Nullable<Integer>,
        from_username -> Text,
        to_username -> Text,
        message_text -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    live_users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        socket_ip -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    rooms (id) {
        id -> Nullable<Integer>,
        name -> Text,
        created_by -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    auth_tokens,
    chat_room_chats,
    direct_msgs,
    live_users,
    rooms,
    users,
);
