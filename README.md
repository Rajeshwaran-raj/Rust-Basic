# MySQL Rust Backend (Axum + SQLx) — Migrations & Seeders on Startup

This backend uses **Axum** (HTTP), **SQLx** (async MySQL), auto-runs **migrations** on boot, and **seeds** initial data if the table is empty.

## Quickstart

1) Install Rust & MySQL, then create a database:
```sql
CREATE DATABASE mysql_backend CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

2) Configure your DB URL (or use the default in `.env`):
```
DATABASE_URL=mysql://root:password@127.0.0.1:3306/mysql_backend
```

3) Run:
```bash
cargo run
# ➜ http://localhost:8080
```

On start:
- Runs migrations in `./migrations`
- Seeds two items if the table is empty

## API

- `GET /health` → `"ok"`
- `GET /items` → list items
- `POST /items` → create `{ "title": "..." }`
- `GET /items/:id` → get one
- `PUT /items/:id` → partial update `{ "title": "...", "done": true }`
- `DELETE /items/:id` → remove

## Stack
- axum 0.7
- sqlx 0.7 (mysql, runtime-tokio-rustls, uuid, chrono)
- tokio 1, serde 1, uuid 1, chrono 0.4
- tower-http 0.5, tracing 0.1

## Notes
- Migrations run with `sqlx::migrate!("./migrations")` at startup.
- Seeder runs once only when `SELECT COUNT(*) FROM items = 0`.
- UUIDs are stored as `CHAR(36)` strings (easy to read/debug).