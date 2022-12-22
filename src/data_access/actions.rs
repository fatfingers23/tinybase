use crate::data_access::models;
use crate::data_access::schema::key_values::dsl::key_values;
use crate::data_access::schema::key_values::dsl::*;
use diesel::prelude::*;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn insert_new_entry(
    conn: &mut SqliteConnection,
    new_key: String,
    new_value: String,
) -> Result<models::NewKeyValue, DbError> {
    let new_key_value = models::NewKeyValue {
        key: new_key,
        value: new_value,
    };

    let _ = diesel::insert_into(key_values)
        .values(&new_key_value)
        .on_conflict(key)
        .do_update()
        .set(&new_key_value)
        .execute(conn);

    Ok(new_key_value)
}

pub fn get_entry(
    conn: &mut SqliteConnection,
    search_key: String,
) -> Result<Option<models::KeyValue>, DbError> {
    let entry = key_values
        .filter(key.eq(search_key))
        .first::<models::KeyValue>(conn)
        .optional()?;

    Ok(entry)
}

pub fn get_keys_by_prefix(conn: &mut SqliteConnection, prefix: String) -> Vec<String> {
    let pattern = format!("{}%", prefix);

    let query_results = key_values
        .filter(key.like(pattern))
        .load::<models::KeyValue>(conn)
        .expect("Could not load the keys");
    let mut results: Vec<String> = Vec::new();

    for entry in query_results {
        results.push(entry.key);
    }
    results
}

pub fn delete_by_key(conn: &mut SqliteConnection, key_to_delete: String) -> bool {
    let num_deleted = diesel::delete(key_values.filter(key.eq(key_to_delete)))
        .execute(conn)
        .expect("Error deleting");

    num_deleted > 0
}
