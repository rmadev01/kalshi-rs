//! API clients for communicating with Kalshi.
//!
//! This module contains:
//!
//! - [`rest`] - HTTP client for REST API endpoints
//! - [`websocket`] - WebSocket client for real-time data
//! - [`auth`] - RSA-PSS authentication utilities

pub mod auth;
pub mod rest;
pub mod websocket;

pub use auth::Signer;
pub use rest::RestClient;
pub use websocket::WebSocketClient;
