// @generated automatically by Diesel CLI.

diesel::table! {
    key_values (id) {
        id -> Nullable<Integer>,
        key -> Text,
        value -> Text,
    }
}
