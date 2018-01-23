#![feature(catch_expr)]

extern crate mattermost_structs;
extern crate serde_json;

use mattermost_structs::websocket::Message;
use serde_json::{Deserializer, Value};
use std::io::stdin;

fn main() {
    println!("Read json lines from stdin...");
    println!();

    let sin = stdin();
    let stream = Deserializer::from_reader(sin.lock()).into_iter::<Value>();

    for value in stream {
        let value = value.unwrap();
        do catch {
            let msg: Result<Message, _> = serde_json::from_value(value.clone());
            if let Err(v) = msg {
                println!("{:?}", v);
                println!("occured while processing {:?}", value);
                println!("\n\n");
            }
        }
    }
}
