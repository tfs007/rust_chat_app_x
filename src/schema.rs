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
    rooms,
    users,
);
