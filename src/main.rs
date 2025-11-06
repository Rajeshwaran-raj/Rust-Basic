#[macro_use] extern crate rocket;
mod state; mod error; mod seed; mod routes;
use dotenvy::dotenv; use rocket_cors::{AllowedHeaders,AllowedOrigins,CorsOptions};
#[launch]
async fn rocket()->_ { dotenv().ok(); let s=state::AppState::new().await.expect("init"); let cors=CorsOptions::default().allowed_origins(AllowedOrigins::all()).allowed_headers(AllowedHeaders::all()).allow_credentials(true).to_cors().unwrap(); rocket::build().manage(s).attach(cors).mount("/", routes![routes::healthz]).mount("/api", routes![routes::users::list_users,routes::users::get_user,routes::users::create_user,routes::users::update_user,routes::users::delete_user,routes::products::list_products,routes::products::create_product,routes::orders::list_orders,routes::orders::create_order]) }
