#![deny(
    rust_2018_compatibility
)]
#![warn(
    rust_2018_idioms,
)]

pub mod api;
pub mod error;
pub use crate::error::{Error, Result};
pub mod websocket;

mod serialize;
