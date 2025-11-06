# Rust Actix-web + MySQL Starter

See endpoints /healthz and /api/users.


# Actix-Web + MySQL Demo

A simple **Rust backend** using **Actix-Web 4** and **SQLx 0.7** with embedded migrations and idempotent seeders.

---

## 🚀 Features

- **Framework:** [Actix-Web 4](https://actix.rs/)
- **Database:** MySQL with [SQLx 0.7](https://docs.rs/sqlx/latest/sqlx/)
- **Migrations:** Embedded using `sqlx::migrate!()`
- **Seeders:** Runs automatically on startup (idempotent)
- **Health Check:** Simple endpoint to verify service health

---

### Run Command
cargo run


## 🧩 Endpoints

| Method | Endpoint | Description |
|:--------|:-----------|:-------------|
| **GET** | `/healthz` | Health check → returns `{ "status": "ok" }` |
| **GET** | `/api/users` | Get all users |
| **POST** | `/api/users` | Create a new user (JSON: `{ "name": "...", "email": "..." }`) |
| **GET** | `/api/users/{id}` | Get user by ID |
| **PUT** | `/api/users/{id}` | Update user by ID |
| **DELETE** | `/api/users/{id}` | Delete user by ID |

---

## ⚙️ Setup Instructions

### 1. Create Database

```sql
CREATE DATABASE actix_demo CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
