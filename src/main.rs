mod state;
mod routes;
mod error;
mod seed;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenvy::dotenv;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let app_state = state::AppState::new().await.expect("init failed");

    let host = std::env::var("APP_HOST").unwrap_or("127.0.0.1".into());
    let port: u16 = std::env::var("APP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    println!("Server http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(Data::new(app_state.clone()))
            .configure(routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}
