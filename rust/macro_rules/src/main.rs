use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

#[derive(Clone, PartialEq, Debug)]
enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>),
}

macro_rules! json {
    (null) => {
        Json::Null
    };
    ([ $( $element:tt ),* ]) => {
        Json::Array(vec![ $( $element ),* ])
    };
    ({ $( $key:tt : $value:tt ),* }) => {
        Json::Object(...)
    };
    ( $other:tt ) => {
        Json::Boolean(...)
    };
    ( $other:tt ) => {
        Json::Number(...)
    };
    ( $other:tt ) => {
        Json::String(...)
    };
}

mod test {
    use super::*;

    #[test]
    fn json_null() {
        assert_eq!(json!(null), Json::Null);
    }

    #[test]
    fn json_array_with_json_element() {
        let macro_generated_value = json!(
            [
                {
                    "pitch": 440.0
                }
            ]
        );
        let hand_coded_value = Json::Array(vec![Json::Object(Box::new(
            vec![("pitch".to_string(), Json::Number(440.0))]
                .into_iter()
                .collect(),
        ))]);
    }
}
