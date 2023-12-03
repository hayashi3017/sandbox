use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Message {
    Request {
        id: String,
        method: String,
        params: Params,
    },
    Response {
        id: String,
        result: Value,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Params {
    param1: u32,
    param2: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Value {
    value1: u32,
    value2: String,
}

// https://serde.rs/enum-representations.html
fn main() {
    println!("Hello, world!");

    let req1 = Message::Request {
        id: String::from_str("id").unwrap(),
        method: String::from_str("method").unwrap(),
        params: Params {
            param1: 1,
            param2: true,
        },
    };
    assert_eq!(serde_json::to_string(&req1).unwrap(), "{\"Request\":{\"id\":\"id\",\"method\":\"method\",\"params\":{\"param1\":1,\"param2\":true}}}");
}
