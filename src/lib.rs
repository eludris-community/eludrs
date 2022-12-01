//! A simple rust library for interacting with the Eludris API
mod gateway;
mod http;
mod models;

pub use gateway::{Events, GatewayClient};
pub use http::HttpClient;

/// All the todel models re-exported
pub mod todel {
    pub use todel::models::*;
}

/// The default rest url
pub const REST_URL: &str = "https://eludris.tooty.xyz";

/// The default gateway url
pub const GATEWAY_URL: &str = "wss://eludris.tooty.xyz/ws/";
