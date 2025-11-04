# In-Memory Rust Backend (Axum)

## Run
```bash
cargo run
# server: http://localhost:8080
```

## Endpoints
- `GET /health` → `"ok"`
- `GET /items` → list
- `POST /items` → create `{ "title": "..." }`
- `GET /items/:id` → get one
- `PUT /items/:id` → partial update `{ "title": "...", "done": true }`
- `DELETE /items/:id` → remove
```bash
curl http://localhost:8080/health
curl -X POST http://localhost:8080/items -H "content-type: application/json" -d '{"title":"first"}'
```

## What the app uses and the exact versions.
```
Stack & Versions

Rust edition: 2021 (works on stable Rust)

Axum: 0.7 (HTTP routing & handlers)

Tokio: 1 (async runtime)

Serde: 1 with derive (JSON (de)serialization)

serde_json: 1

uuid: 1 with features v4,serde (IDs)

thiserror: 1 (error ergonomics)

tower-http: 0.5 with features cors,trace (CORS + request tracing)

tracing: 0.1 (structured logs)

tracing-subscriber: 0.3 with env-filter (log formatting & filtering)

What’s used in the code (“the things”)

Framework: Axum router with GET/POST/PUT/DELETE routes.

Runtime: Tokio async (#[tokio::main]).

Storage: In-memory HashMap<Uuid, Item> guarded by Arc<RwLock<...>>.

Model: Item { id, title, done }.

JSON: Serde for request/response bodies.

IDs: Uuid::new_v4().

Middleware: tower_http::CorsLayer (open CORS) and TraceLayer (request logs).

Logging: tracing + tracing-subscriber (env-configurable).

Port: 0.0.0.0:8080.

Health check: GET /health.
```