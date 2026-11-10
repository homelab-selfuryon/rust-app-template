//! An app on the homelab-selfuryon paved road.
//!
//! - [`config`]: non-secret settings from the environment
//! - [`secrets`]: secrets from files, wrapped in `secrecy`
//! - [`auth`]: verification of projected ServiceAccount tokens
//! - [`routes`]: the HTTP routes; `/healthz` and `/readyz` for the kubelet

pub mod auth;
pub mod config;
pub mod routes;
pub mod secrets;
pub mod state;

pub use state::AppState;
