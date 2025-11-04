use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, FromRow, MySql, Pool};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: Pool<MySql>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
struct Item {
    id: String, // store UUID as CHAR(36)
    title: String,
    done: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateItem {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateItem {
    title: Option<String>,
    done: Option<bool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logging setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "mysql-backend=debug,tower_http=debug,axum=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // environment and DB setup
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mysql://root:password@127.0.0.1:3306/mysql_backend".to_string()
    });

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // run migrations
    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations complete.");

    // seed data
    seed_if_empty(&pool).await?;

    let state = AppState { pool };

    // routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/items", get(list_items).post(create_item))
        .route("/items/:id", get(get_item).put(update_item).delete(delete_item))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        )
        .layer(TraceLayer::new_for_http());

    // ✅ Axum 0.7 style server start
    use tokio::net::TcpListener;
    use axum::serve;

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("🚀 MySQL API listening on http://{addr}");
    serve(listener, app).await?;
    Ok(())
}

async fn seed_if_empty(pool: &Pool<MySql>) -> anyhow::Result<()> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM items")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        tracing::info!("Seeding initial data...");
        let now = Utc::now();
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO items (id, title, done, created_at) VALUES (?, ?, ?, ?)")
            .bind(&id1)
            .bind("Try the API")
            .bind(false)
            .bind(now)
            .execute(pool)
            .await?;

        sqlx::query("INSERT INTO items (id, title, done, created_at) VALUES (?, ?, ?, ?)")
            .bind(&id2)
            .bind("Write something awesome")
            .bind(false)
            .bind(now)
            .execute(pool)
            .await?;
    } else {
        tracing::info!("Seed skipped (items table already has data).");
    }
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

// GET /items
async fn list_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let rows: Vec<Item> = sqlx::query_as::<_, Item>(
        "SELECT id, title, done, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_err)?;
    Ok(Json(rows))
}

// POST /items { "title": "..." }
async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query("INSERT INTO items (id, title, done, created_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&payload.title)
        .bind(false)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(internal_err)?;

    let created: Item =
        sqlx::query_as("SELECT id, title, done, created_at FROM items WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
            .map_err(internal_err)?;

    Ok((StatusCode::CREATED, Json(created)))
}

// GET /items/:id
async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let item: Item =
        sqlx::query_as("SELECT id, title, done, created_at FROM items WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
            .map_err(not_found_or_internal)?;
    Ok(Json(item))
}

// PUT /items/:id
async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateItem>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let mut item: Item =
        sqlx::query_as("SELECT id, title, done, created_at FROM items WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
            .map_err(not_found_or_internal)?;

    if let Some(t) = payload.title {
        item.title = t;
    }
    if let Some(d) = payload.done {
        item.done = d;
    }

    sqlx::query("UPDATE items SET title = ?, done = ? WHERE id = ?")
        .bind(&item.title)
        .bind(item.done)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(internal_err)?;

    Ok(Json(item))
}

// DELETE /items/:id
async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(internal_err)?;

    if res.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Item not found".into()))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {e}"),
    )
}

fn not_found_or_internal(e: sqlx::Error) -> (StatusCode, String) {
    match e {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Item not found".into()),
        other => internal_err(other),
    }
}
