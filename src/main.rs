use axum::{
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    cache: Arc<Mutex<HashMap<String, Value>>>,
    limits: Arc<Mutex<HashMap<String, Window>>>,
    config: Config,
}

#[derive(Clone)]
struct Config {
    max_requests: u32,
    window: Duration,
}

#[derive(Clone)]
struct Window {
    started: Instant,
    requests: u32,
}

#[derive(Debug, Deserialize)]
struct CacheQuery {
    key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheEntry {
    value: Value,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    cached_entries: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "tidegate=info".into()))
        .init();

    let state = AppState::from_env();
    let address = env::var("TIDEGATE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let socket: SocketAddr = address.parse().expect("TIDEGATE_ADDR must be host:port");

    let app = app(state);
    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .expect("bind server");
    info!(address = %socket, "tidegate listening");
    axum::serve(listener, app).await.expect("serve application");
}

fn app(state: AppState) -> Router {
    let rate_state = state.clone();
    Router::new()
        .route("/healthz", get(health))
        .route("/v1/cache", get(read_cache))
        .route("/v1/cache/:key", put(write_cache).delete(delete_cache))
        .with_state(state)
        .layer(middleware::from_fn_with_state(rate_state, rate_limit))
        .layer(TraceLayer::new_for_http())
}

impl AppState {
    fn from_env() -> Self {
        let max_requests = env::var("TIDEGATE_RATE_LIMIT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(60);
        let window_seconds = env::var("TIDEGATE_RATE_WINDOW_SECONDS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(60);

        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            limits: Arc::new(Mutex::new(HashMap::new())),
            config: Config {
                max_requests,
                window: Duration::from_secs(window_seconds),
            },
        }
    }
}

async fn rate_limit(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let client = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("local")
        .split(',')
        .next()
        .unwrap_or("local")
        .trim()
        .to_owned();

    let now = Instant::now();
    let allowed = {
        let mut limits = state.limits.lock().expect("rate limiter lock");
        let window = limits.entry(client).or_insert(Window {
            started: now,
            requests: 0,
        });
        if now.duration_since(window.started) >= state.config.window {
            window.started = now;
            window.requests = 0;
        }
        window.requests += 1;
        window.requests <= state.config.max_requests
    };

    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", state.config.window.as_secs().to_string())],
            Json(json!({ "error": "rate limit exceeded" })),
        )
            .into_response();
    }

    next.run(request).await
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let cached_entries = state.cache.lock().expect("cache lock").len();
    Json(HealthResponse {
        status: "ok",
        cached_entries,
    })
}

async fn read_cache(
    State(state): State<AppState>,
    Query(query): Query<CacheQuery>,
) -> Result<Json<CacheEntry>, (StatusCode, Json<Value>)> {
    let value = state
        .cache
        .lock()
        .expect("cache lock")
        .get(&query.key)
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "cache key not found" })),
            )
        })?;
    Ok(Json(CacheEntry { value }))
}

async fn write_cache(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(entry): Json<CacheEntry>,
) -> (StatusCode, Json<CacheEntry>) {
    state
        .cache
        .lock()
        .expect("cache lock")
        .insert(key, entry.value.clone());
    (StatusCode::CREATED, Json(entry))
}

async fn delete_cache(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state
        .cache
        .lock()
        .expect("cache lock")
        .remove(&key)
        .is_some()
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_round_trip_is_json_safe() {
        let state = AppState::from_env();
        state
            .cache
            .lock()
            .unwrap()
            .insert("demo".into(), json!({"enabled": true}));
        assert_eq!(
            state.cache.lock().unwrap().get("demo"),
            Some(&json!({"enabled": true}))
        );
    }

    #[test]
    fn default_config_is_conservative() {
        let state = AppState::from_env();
        assert_eq!(state.config.max_requests, 60);
        assert_eq!(state.config.window, Duration::from_secs(60));
    }
}
