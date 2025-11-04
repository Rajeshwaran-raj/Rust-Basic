use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Item {
    id: Uuid,
    title: String,
    done: bool,
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

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<Uuid, Item>>>,
}

#[tokio::main]
async fn main() {
    // logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inmemory-backend=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

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

    // ✅ New Axum 0.7+ style server setup
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("🚀 In-memory API listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

// GET /items
async fn list_items(State(state): State<AppState>) -> Json<Vec<Item>> {
    let guard = state.store.read().await;
    Json(guard.values().cloned().collect())
}

// POST /items { "title": "..." }
async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<Item>) {
    let item = Item {
        id: Uuid::new_v4(),
        title: payload.title,
        done: false,
    };
    {
        let mut guard = state.store.write().await;
        guard.insert(item.id, item.clone());
    }
    (StatusCode::CREATED, Json(item))
}

// GET /items/:id
async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let guard = state.store.read().await;
    guard
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Item not found".into()))
}

// PUT /items/:id { "title": "...", "done": true }
async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItem>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let mut guard = state.store.write().await;
    let item = guard
        .get_mut(&id)
        .ok_or((StatusCode::NOT_FOUND, "Item not found".into()))?;
    if let Some(t) = payload.title {
        item.title = t;
    }
    if let Some(d) = payload.done {
        item.done = d;
    }
    Ok(Json(item.clone()))
}

// DELETE /items/:id
async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut guard = state.store.write().await;
    guard
        .remove(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or((StatusCode::NOT_FOUND, "Item not found".into()))
}
