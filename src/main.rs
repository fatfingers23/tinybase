mod actions;
mod actors;
mod controllers;
mod models;
mod schema;

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use actors::{session::WsChatSession, ws_actor::ClientWebSocketConnection, ws_actor::Test};
use controllers::key_controller::{create_key, get_key, list_keys, url_create_key};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use uuid::Uuid;
extern crate dotenv;
//extern crate urlencoding;

mod auth_middleware;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ClientWebSocketConnection>>,
) -> Result<HttpResponse, Error> {
    println!();
    ws::start(
        WsChatSession {
            id: Uuid::new_v4(),
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

/// Displays state
async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}

/// Displays state
async fn test(addr: web::Data<Addr<ClientWebSocketConnection>>) -> impl Responder {
    let result = addr.send(Test {}).await;
    return match result {
        Ok(test) => return format!("{}", test),
        Err(_error) => {
            return format!("{}", _error);
        }
    };
}

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv::dotenv().ok();
    // set up applications state
    // keep a count of the number of visitors
    let app_state = Arc::new(AtomicUsize::new(0));
    // start chat server actor
    let server = ClientWebSocketConnection::new(app_state.clone()).start();

    // set up database connection pool
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    log::info!("starting HTTP server at http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(Files::new("/static", "./static"))
            .service(web::resource("/").to(index))
            .service(
                web::scope("/v0/{secret}")
                    .route("/ws", web::get().to(chat_route))
                    .service(url_create_key)
                    .service(create_key)
                    .service(get_key)
                    .service(list_keys)
                    .route("/test", web::get().to(test))
                    .wrap(auth_middleware::CheckForSecret),
            )
            .route("/count", web::get().to(get_count))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
