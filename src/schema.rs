// @generated automatically by Diesel CLI.

diesel::table! {
    rooms (id) {
        id -> Nullable<Integer>,
        name -> Text,
        created_by -> Text,
        created_at -> Timestamp,
    }
}
