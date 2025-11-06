use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateProduct {
    pub name: String,
    pub price_cents: i64,
}

#[get("/products")]
pub async fn list_products(s: &State<AppState>) -> Result<Json<Vec<Product>>, ApiError> {
    let rows: Vec<Product> = sqlx::query_as(
        "SELECT id, name, price_cents, created_at, updated_at
         FROM products
         ORDER BY created_at DESC",
    )
    .fetch_all(&s.db)
    .await?;

    Ok(Json(rows))
}

#[post("/products", data = "<p>")]
pub async fn create_product(
    s: &State<AppState>,
    p: Json<CreateProduct>,
) -> Result<Json<Product>, ApiError> {
    if p.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }
    if p.price_cents < 0 {
        return Err(ApiError::BadRequest("price_cents must be >= 0".into()));
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO products (id, name, price_cents, created_at, updated_at)
         VALUES (?,?,?,?,?)",
    )
    .bind(id)
    .bind(&p.name)
    .bind(p.price_cents)
    .bind(now)
    .bind(now)
    .execute(&s.db)
    .await?;

    let created: Product = sqlx::query_as(
        "SELECT id, name, price_cents, created_at, updated_at
         FROM products
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await?;

    Ok(Json(created))
}
