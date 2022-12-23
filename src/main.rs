use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

mod actors;
mod auth_middleware;
mod controllers;
mod data_access;
mod migrations;
use std::{
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
use actors::{session::WsChatSession, ws_actor::ClientWebSocketConnection};
use controllers::key_controller::*;
use diesel::r2d2::ManageConnection;
// extern crate diesel_migrations;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use std::env;

use uuid::Uuid;

extern crate dotenv;
//extern crate urlencoding;

// pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ClientWebSocketConnection>>,
) -> Result<HttpResponse, Error> {
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

// /// Example on how to send an actor a message from a "controller"
// async fn test(addr: web::Data<Addr<ClientWebSocketConnection>>) -> impl Responder {
//     let result = addr.send(Test {}).await;
//     return match result {
//         Ok(test) => return format!("{}", test),
//         Err(_error) => {
//             return format!("{}", _error);
//         }
//     };
// }

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
    let conn_spec = match env::var("DATABASE_URL") {
        Ok(db_name) => db_name,
        Err(_) => "tinybase.db".to_string(),
    };

    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let mut conn = pool.get().expect("Could not get instance of the DB");
    migrations::run(&mut conn).unwrap();
    drop(conn);

    let port: u16 = match env::var("DB_PORT") {
        Ok(unwrapped_port) => unwrapped_port.parse().unwrap(),
        Err(_) => 8080,
    };

    let host = match env::var("DB_HOST") {
        Ok(unwrapped_host) => unwrapped_host,
        Err(_) => "0.0.0.0".to_string(),
    };

    log::info!("starting HTTP server at http://{host}:{port}");

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
                    .service(delete_key)
                    .wrap(auth_middleware::CheckForSecret),
            )
            .route("/count", web::get().to(get_count))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind((host, port))?
    .run()
    .await
}
