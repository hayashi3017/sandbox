use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct W {
    a: i32,
    b: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct X(i32, i32);

#[derive(Debug, Serialize, Deserialize)]
struct Y(i32);

#[derive(Debug, Serialize, Deserialize)]
struct Z;

#[derive(Debug, Serialize, Deserialize)]
enum E {
    W { a: i32, b: i32 },
    X(i32, i32),
    Y(i32),
    Z,
}

fn main() {
    println!("Hello, world!");
}

// https://serde.rs/json.html
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_serialized() {
        let w = W { a: 0, b: 0 };
        assert_eq!(
            serde_json::to_string(&w).unwrap(),
            "{\"a\":0,\"b\":0}".to_string()
        );

        let x = X(0, 0);
        assert_eq!(serde_json::to_string(&x).unwrap(), "[0,0]".to_string());

        let y = Y(0);
        assert_eq!(serde_json::to_string(&y).unwrap(), "0".to_string());

        // let z: Z;
        // assert_eq!(serde_json::to_vec(z).unwrap(), "null".to_string());

        let ew = E::W { a: 0, b: 0 };
        assert_eq!(serde_json::to_string(&ew).unwrap(), "{\"W\":{\"a\":0,\"b\":0}}".to_string());
        let ex = E::X(0, 0);
        assert_eq!(serde_json::to_string(&ex).unwrap(), "{\"X\":[0,0]}".to_string());
        let ey = E::Y(0);
        assert_eq!(serde_json::to_string(&ey).unwrap(), "{\"Y\":0}".to_string());
        // let ez = E::Z;
    }
}
