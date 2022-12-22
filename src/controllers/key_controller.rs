//use urlencoding::encode;
use crate::actions::{get_entry, insert_new_entry};
use actix_web::web;
use actix_web::{
    delete, get, post,
    web::{block, Bytes, Data, Form, Json, Path, Query},
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
    println!("from request {}", body);
    let body_split: Vec<&str> = body.split("=").collect();
    let key = body_split.get(0);
    let value = body_split.get(1);
    println!("Whart {}", value.unwrap());
    if let (Some(unwraped_key), Some(unwraped_value)) = (key, value) {
        let decoded_key = decode(unwraped_key).to_owned().unwrap().to_string();
        let decoded_value = decode(unwraped_value).to_owned().unwrap().to_string();

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

            //Need to like encoded or decode or somthing
            return HttpResponse::Ok().body(format!("{}", value));
        }
        Err(_) => HttpResponse::Ok().body(format!("")),
    }
}

#[delete("/{key}")]
pub async fn delete_key(params: Path<KeyPath>) -> HttpResponse {
    let params = params.into_inner();
    let key = params.key;
    //Need to return no content if its foudn and deleted, 404 if its not found
    return HttpResponse::NoContent().body(format!("{}", key));
}

#[derive(Deserialize)]
pub struct KeyList {
    prefix: String,
    encode: Option<bool>,
}

#[get("")]
pub async fn list_keys(params: Query<KeyList>) -> HttpResponse {
    let params = params.into_inner();
    let encode_keys = match params.encode {
        Some(param_encode) => param_encode,
        None => false,
    };

    let pretend_keys = ["test", "test2", "{'test': 'test}", "pl a a "];
    println!("{}", encode_keys);
    match encode_keys {
        true => {
            let mut enocded_keys: Vec<String> = Vec::new();
            for key in pretend_keys {
                let encoded_key = encode(key).into_owned();
                enocded_keys.push(encoded_key);
            }
            let res = enocded_keys.join("\n");
            print!("Encode truye");
            return HttpResponse::Ok().body(format!("{}", res));
        }
        false => {
            let res = pretend_keys.join("\n");

            return HttpResponse::Ok().body(format!("{}", res));
        }
    }
}
