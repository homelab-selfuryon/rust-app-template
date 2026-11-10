//! The HTTP surface, through the router without a socket.

mod common;

use std::{collections::HashMap, sync::Arc, sync::atomic::Ordering};

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{AUDIENCE, CALLER, StaticJwks, TestKey};
use http_body_util::BodyExt;
use rust_app_template::{
    AppState,
    auth::{Verifier, VerifierOptions},
    config::Config,
    routes,
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn app(auth: Option<&StaticJwks>) -> Result<(Router, AppState)> {
    let map: HashMap<String, String> = HashMap::new();
    let config = Config::from_lookup(|k| map.get(k).cloned())?;
    let verifier = auth.map(|s| {
        Verifier::new(
            VerifierOptions::new(AUDIENCE.to_owned(), [CALLER.to_owned()]),
            Arc::new(s.clone()),
        )
    });
    let state = AppState::new(config, verifier);
    Ok((routes::router(state.clone()), state))
}

async fn get(app: &Router, uri: &str, bearer: Option<&str>) -> Result<(StatusCode, Value)> {
    let mut req = Request::builder().uri(uri);
    if let Some(t) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let res = app.clone().oneshot(req.body(Body::empty())?).await?;
    let status = res.status();
    let bytes = res.into_body().collect().await?.to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok((status, body))
}

#[tokio::test]
async fn health_and_readiness() -> Result<()> {
    let (app, state) = app(None)?;
    assert_eq!(get(&app, "/healthz", None).await?.0, StatusCode::OK);
    assert_eq!(get(&app, "/readyz", None).await?.0, StatusCode::OK);
    state.ready.store(false, Ordering::Relaxed);
    assert_eq!(
        get(&app, "/readyz", None).await?.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(get(&app, "/healthz", None).await?.0, StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn public_route() -> Result<()> {
    let (app, _) = app(None)?;
    let (status, body) = get(&app, "/v1/hello", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["message"], "hello");
    Ok(())
}

#[tokio::test]
async fn protected_route_without_auth_configured_is_closed() -> Result<()> {
    let (app, _) = app(None)?;
    let (status, _) = get(&app, "/v1/whoami", Some("anything")).await?;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test]
async fn protected_route_statuses() -> Result<()> {
    let key = TestKey::new("k1")?;
    let (app, _) = app(Some(&StaticJwks::new(&[&key])))?;

    let (status, _) = get(&app, "/v1/whoami", None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no token");

    let (status, _) = get(&app, "/v1/whoami", Some("junk")).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "bad token");

    let intruder = key.token(json!({"sub": "system:serviceaccount:other:x"}))?;
    let (status, _) = get(&app, "/v1/whoami", Some(&intruder)).await?;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "valid token, subject not allowed"
    );

    let (status, body) = get(&app, "/v1/whoami", Some(&key.token(json!({}))?)).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["caller"], CALLER);
    Ok(())
}
