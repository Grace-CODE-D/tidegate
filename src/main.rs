use axum::{
    body::Body,
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
    collections::{HashMap, VecDeque},
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    cache: Arc<Mutex<CacheStore>>,
    limits: Arc<Mutex<HashMap<String, Window>>>,
    metrics: Arc<Mutex<Metrics>>,
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

struct CacheStore {
    entries: HashMap<String, Value>,
    insertion_order: VecDeque<String>,
    capacity: usize,
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
    cache_capacity: usize,
}

#[derive(Clone, Default, Serialize)]
struct Metrics {
    total_requests: u64,
    rejected_requests: u64,
    cache_reads: u64,
    cache_writes: u64,
    cache_deletes: u64,
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
        .route("/metrics", get(metrics))
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
        let cache_capacity = env::var("TIDEGATE_CACHE_CAPACITY")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(256);

        Self {
            cache: Arc::new(Mutex::new(CacheStore::new(cache_capacity))),
            limits: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Metrics::default())),
            config: Config {
                max_requests,
                window: Duration::from_secs(window_seconds),
            },
        }
    }
}

impl CacheStore {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "cache capacity must be greater than zero");
        Self {
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
            capacity,
        }
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key)
    }

    fn insert(&mut self, key: String, value: Value) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), value);
            self.insertion_order.retain(|entry| entry != &key);
            self.insertion_order.push_back(key);
            return;
        }

        if self.entries.len() == self.capacity {
            if let Some(oldest) = self.insertion_order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
        self.insertion_order.push_back(key.clone());
        self.entries.insert(key, value);
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        let value = self.entries.remove(key)?;
        self.insertion_order.retain(|entry| entry != key);
        Some(value)
    }
}

async fn rate_limit(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
    state.metrics.lock().expect("metrics lock").total_requests += 1;
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
        state
            .metrics
            .lock()
            .expect("metrics lock")
            .rejected_requests += 1;
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
    let cache = state.cache.lock().expect("cache lock");
    Json(HealthResponse {
        status: "ok",
        cached_entries: cache.entries.len(),
        cache_capacity: cache.capacity,
    })
}

async fn metrics(State(state): State<AppState>) -> Json<Metrics> {
    Json(state.metrics.lock().expect("metrics lock").clone())
}

async fn read_cache(
    State(state): State<AppState>,
    Query(query): Query<CacheQuery>,
) -> Result<Json<CacheEntry>, (StatusCode, Json<Value>)> {
    state.metrics.lock().expect("metrics lock").cache_reads += 1;
    let cache = state.cache.lock().expect("cache lock");
    let value = cache.get(&query.key).cloned().ok_or_else(|| {
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
    state.metrics.lock().expect("metrics lock").cache_writes += 1;
    state
        .cache
        .lock()
        .expect("cache lock")
        .insert(key, entry.value.clone());
    (StatusCode::CREATED, Json(entry))
}

async fn delete_cache(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    state.metrics.lock().expect("metrics lock").cache_deletes += 1;
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
    use axum::body::to_bytes;
    use tower::ServiceExt;

    fn test_state(capacity: usize) -> AppState {
        AppState {
            cache: Arc::new(Mutex::new(CacheStore::new(capacity))),
            limits: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Metrics::default())),
            config: Config {
                max_requests: 60,
                window: Duration::from_secs(60),
            },
        }
    }

    #[test]
    fn cache_round_trip_is_json_safe() {
        let mut cache = CacheStore::new(2);
        cache.insert("demo".into(), json!({"enabled": true}));
        assert_eq!(cache.get("demo"), Some(&json!({"enabled": true})));
    }

    #[test]
    fn cache_evicts_the_oldest_entry_at_capacity() {
        let mut cache = CacheStore::new(2);
        cache.insert("first".into(), json!(1));
        cache.insert("second".into(), json!(2));
        cache.insert("third".into(), json!(3));

        assert!(cache.get("first").is_none());
        assert_eq!(cache.get("second"), Some(&json!(2)));
        assert_eq!(cache.get("third"), Some(&json!(3)));
    }

    #[test]
    fn updating_a_key_does_not_consume_capacity() {
        let mut cache = CacheStore::new(2);
        cache.insert("first".into(), json!(1));
        cache.insert("second".into(), json!(2));
        cache.insert("first".into(), json!(10));
        cache.insert("third".into(), json!(3));

        assert_eq!(cache.get("first"), Some(&json!(10)));
        assert!(cache.get("second").is_none());
    }

    #[test]
    fn default_config_is_conservative() {
        let state = AppState::from_env();
        assert_eq!(state.config.max_requests, 60);
        assert_eq!(state.config.window, Duration::from_secs(60));
        assert_eq!(state.cache.lock().unwrap().capacity, 256);
    }

    #[tokio::test]
    async fn cache_routes_work_end_to_end() {
        let app = app(test_state(2));
        let request = Request::builder()
            .method("PUT")
            .uri("/v1/cache/demo")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"value":{"message":"hello"}}"#))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .uri("/v1/cache?key=demo")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, r#"{"value":{"message":"hello"}}"#);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            body,
            r#"{"total_requests":3,"rejected_requests":0,"cache_reads":1,"cache_writes":1,"cache_deletes":0}"#
        );
    }
}
