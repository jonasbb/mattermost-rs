#![feature(never_type)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_with;
extern crate serde_yaml;
extern crate url;

pub mod api;
pub mod error;
pub use error::{Error, Result};
pub mod websocket;

mod serialize;
