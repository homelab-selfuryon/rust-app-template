use std::sync::atomic::Ordering;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::{Value, json};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::{AppState, auth::Caller};

/// All routes. `/healthz` and `/readyz` are for the kubelet; replace
/// `/v1/hello` and `/v1/whoami` with the app's own.
pub fn router(state: AppState) -> Router {
    let timeout = state.config.request_timeout;
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/hello", get(hello))
        .route("/v1/whoami", get(whoami))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            timeout,
        ))
        .with_state(state)
}

/// Liveness: the process answers.
async fn healthz() -> &'static str {
    "ok"
}

/// Readiness: fails while shutting down so the endpoint leaves the Service.
async fn readyz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.ready.load(Ordering::Relaxed) {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "shutting down")
    }
}

async fn hello(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "message": "hello",
        "env": state.config.app_env,
        "version": state.config.app_version,
    }))
}

/// An authenticated route: only allowed ServiceAccounts get in.
async fn whoami(Caller(subject): Caller) -> Json<Value> {
    Json(json!({ "caller": subject }))
}
