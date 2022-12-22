//use urlencoding::encode;
use crate::data_access::actions::*;
use actix_web::web;
use actix_web::{
    delete, get, post,
    web::{block, Path, Query},
    HttpResponse,
};
use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use serde::Deserialize;
use urlencoding::{decode, encode};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[derive(Deserialize)]
pub struct KeyValue {
    key: String,
    value: String,
}

#[post("/{key}={value}")]
pub async fn url_create_key(pool: web::Data<DbPool>, info: Path<KeyValue>) -> HttpResponse {
    let key_value = info.into_inner();
    let key = key_value.key;
    let value = key_value.value;
    let _ = block(move || {
        let mut conn = pool.get().expect("Could not get instance of the DB");
        insert_new_entry(&mut conn, key, value)
    })
    .await;
    HttpResponse::Ok().body(format!(""))
}

#[post("")]
pub async fn create_key(pool: web::Data<DbPool>, body: String) -> HttpResponse {
    let body_split: Vec<&str> = body.split("=").collect();
    let key = body_split.get(0);
    let value = body_split.get(1);
    if let (Some(unwrapped_key), Some(unwrapped_value)) = (key, value) {
        let decoded_key = decode(unwrapped_key).to_owned().unwrap().to_string();
        let decoded_value = decode(unwrapped_value).to_owned().unwrap().to_string();
        let _ = block(move || {
            let mut conn = pool.get().expect("Could not get instance of the DB");
            insert_new_entry(&mut conn, decoded_key, decoded_value)
        })
        .await;
        return HttpResponse::Ok().body(format!(""));
    } else {
        return HttpResponse::BadRequest().into();
    }
}

#[derive(Deserialize)]
pub struct KeyPath {
    key: String,
}

#[get("/{key}")]
pub async fn get_key(pool: web::Data<DbPool>, params: Path<KeyPath>) -> HttpResponse {
    let params = params.into_inner();
    let key = params.key;
    let entry = web::block(move || {
        let mut conn = pool.get()?;
        get_entry(&mut conn, key)
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError);

    match entry {
        Ok(unwrapped_entry) => {
            let value = unwrapped_entry.unwrap().unwrap().value;
            return HttpResponse::Ok().body(format!("{}", value));
        }
        Err(_) => HttpResponse::Ok().body(format!("")),
    }
}

#[delete("/{key}")]
pub async fn delete_key(pool: web::Data<DbPool>, params: Path<KeyPath>) -> HttpResponse {
    let params = params.into_inner();
    let delete_results = web::block(move || {
        let mut conn = pool.get().unwrap();
        delete_by_key(&mut conn, params.key)
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError);

    match delete_results {
        Ok(did_it_delete) => match did_it_delete {
            true => HttpResponse::NoContent().body(format!("")),
            false => HttpResponse::NotFound().body(format!("")),
        },
        Err(_) => HttpResponse::BadRequest().body(format!("")),
    }
}

#[derive(Deserialize)]
pub struct KeyList {
    prefix: Option<String>,
    encode: Option<bool>,
}

#[get("")]
pub async fn list_keys(pool: web::Data<DbPool>, params: Query<KeyList>) -> HttpResponse {
    let params = params.into_inner();

    let prefix = match params.prefix {
        Some(param_prefix) => param_prefix,
        None => return HttpResponse::Ok().body(format!("")),
    };
    let encode_keys = match params.encode {
        Some(param_encode) => param_encode,
        None => false,
    };

    let results = web::block(move || {
        let mut conn = pool.get().unwrap();
        get_keys_by_prefix(&mut conn, prefix)
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError);

    match results {
        Ok(keys) => match encode_keys {
            true => {
                let mut encoded_keys: Vec<String> = Vec::new();
                for key in keys {
                    let encoded_key = encode(key.as_str()).into_owned();
                    encoded_keys.push(encoded_key);
                }
                let res = encoded_keys.join("\n");
                print!("Encode truye");
                return HttpResponse::Ok().body(format!("{}", res));
            }
            false => {
                let res = keys.join("\n");

                return HttpResponse::Ok().body(format!("{}", res));
            }
        },
        Err(_) => HttpResponse::Ok().body(format!("")),
    }
}
