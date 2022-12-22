use crate::models;
use diesel::prelude::*;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn insert_new_entry(
    conn: &mut SqliteConnection,
    new_key: String,
    new_value: String,
) -> Result<models::NewKeyValue, DbError> {
    use crate::schema::key_values::dsl::*;

    let new_key_value = models::NewKeyValue {
        key: new_key,
        value: new_value,
    };

    _ = diesel::insert_into(key_values)
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
    use crate::schema::key_values::dsl::*;

    let entry = key_values
        .filter(key.eq(search_key))
        .first::<models::KeyValue>(conn)
        .optional()?;

    Ok(entry)
}

pub fn get_keys_by_prefix(conn: &mut SqliteConnection, prefix: String) {}
