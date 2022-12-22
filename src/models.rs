use diesel::{Queryable, Insertable, AsChangeset};
use serde::{Deserialize, Serialize};

use crate::schema::key_values;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(primary_key(id))]
#[diesel(table_name = key_values)]
pub struct KeyValue {
    pub id: Option<i32>,
    pub key: String,
    pub value: String
}

#[derive(Debug, Insertable, AsChangeset )]
#[diesel(table_name = key_values)]
pub struct NewKeyValue {
    pub key: String,
    pub value: String
}