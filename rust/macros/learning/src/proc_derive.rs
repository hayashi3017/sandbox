//! # Deriveマクロ（`#[derive(...)]`）
//!
//! Deriveマクロは`#[derive(TraitName)]`アノテーションで
//! 型に対して自動実装を生成する手続きマクロです。
//!
//! ## 標準ライブラリのDeriveマクロ
//!
//! | マクロ | 効果 |
//! |---|---|
//! | `Debug` | `{:?}`フォーマットを実装 |
//! | `Clone` | `.clone()`を実装 |
//! | `Copy` | コピーセマンティクスを有効化 |
//! | `PartialEq` / `Eq` | `==`, `!=`を実装 |
//! | `PartialOrd` / `Ord` | `<`, `>`等を実装 |
//! | `Hash` | `HashMap`のキーとして使用可能 |
//! | `Default` | `Default::default()`を実装 |
//!
//! ## このモジュールのカスタムDeriveマクロ（`macros_impl`クレート）
//!
//! | マクロ | 効果 |
//! |---|---|
//! | `Describe` | `describe()`メソッドを生成（フィールド名と値を文字列化） |
//! | `Builder` | ビルダーパターンを生成（`FooBuilder::builder()` → `.field()` → `.build()`) |
//! | `IntoHashMap` | `into_hashmap()`を生成（`HashMap<String, String>`に変換） |
//! | `DefaultNew` | `new()`を生成（`Default::default()`を呼ぶ） |

use macros_impl::{Builder, DefaultNew, Describe, IntoHashMap};

// ─── #[derive(Describe)] ─────────────────────────────────────────────────────

/// `Describe`deriveのテスト用構造体（named struct）
#[derive(Describe, Debug)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: String,
}

/// `Describe`deriveのテスト用構造体（tuple struct）
#[derive(Describe, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// ユニット構造体にも適用可能
#[derive(Describe, Debug)]
pub struct Unit;

// ─── #[derive(Builder)] ──────────────────────────────────────────────────────

/// `Builder`deriveのテスト用構造体
#[derive(Builder, Debug, Clone, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub timeout_ms: u64,
}

/// オプションフィールドを含む構造体のビルダー
#[derive(Builder, Debug, Clone, PartialEq)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
}

// ─── #[derive(IntoHashMap)] ───────────────────────────────────────────────────

/// `IntoHashMap`deriveのテスト用構造体
#[derive(IntoHashMap, Debug)]
pub struct Metrics {
    pub requests: u64,
    pub errors: u64,
    pub latency_ms: f64,
}

// ─── #[derive(DefaultNew)] ───────────────────────────────────────────────────

/// `DefaultNew`deriveのテスト用構造体
#[derive(DefaultNew, Default, Debug, PartialEq)]
pub struct Counter {
    pub value: i32,
    pub name: String,
}

// ─── 標準Deriveマクロの組み合わせ ──────────────────────────────────────────

/// 複数のDeriveを組み合わせた実用的な例
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UserId(pub u64);

/// 比較可能な値オブジェクト
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Score {
    pub value: f64,
    pub label: String,
}

impl Score {
    pub fn new(value: f64, label: impl Into<String>) -> Self {
        Score {
            value,
            label: label.into(),
        }
    }
}

/// Hashを実装した複合キー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub namespace: String,
    pub key: String,
}

impl CacheKey {
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        CacheKey {
            namespace: namespace.into(),
            key: key.into(),
        }
    }
}

// ─── Deriveのderive（Derive for derived types）───────────────────────────────

/// Deriveを通じてトレイト実装を引き継ぐ例
///
/// `#[derive(Clone)]`は各フィールドに`Clone`が実装されている場合に自動生成される。
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapper<T: Clone + std::fmt::Debug + PartialEq> {
    pub inner: T,
    pub metadata: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_describe_named_struct() {
        let p = Person {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };
        let desc = p.describe();
        assert!(desc.contains("Person"));
        assert!(desc.contains("name: \"Alice\""));
        assert!(desc.contains("age: 30"));
        assert!(desc.contains("email: \"alice@example.com\""));
    }

    #[test]
    fn test_describe_tuple_struct() {
        let rgb = Rgb(255, 128, 0);
        let desc = rgb.describe();
        assert!(desc.contains("Rgb"));
        assert!(desc.contains("255"));
        assert!(desc.contains("128"));
    }

    #[test]
    fn test_describe_unit_struct() {
        let u = Unit;
        let desc = u.describe();
        assert!(desc.contains("Unit"));
    }

    #[test]
    fn test_builder_success() {
        let config = ServerConfig::builder()
            .host("localhost".to_string())
            .port(8080)
            .max_connections(100)
            .timeout_ms(5000)
            .build()
            .unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_builder_missing_field() {
        let result = ServerConfig::builder()
            .host("localhost".to_string())
            .port(8080)
            // max_connections, timeout_msは未設定
            .build();

        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("required"), "Error: {}", msg);
    }

    #[test]
    fn test_builder_overwrite_field() {
        let config = ServerConfig::builder()
            .host("old".to_string())
            .host("new".to_string()) // 上書き
            .port(80)
            .max_connections(10)
            .timeout_ms(1000)
            .build()
            .unwrap();

        assert_eq!(config.host, "new");
    }

    #[test]
    fn test_http_request_builder() {
        let req = HttpRequest::builder()
            .method("GET".to_string())
            .url("https://example.com".to_string())
            .body(None)
            .build()
            .unwrap();

        assert_eq!(req.method, "GET");
        assert!(req.body.is_none());
    }

    #[test]
    fn test_into_hashmap() {
        let m = Metrics {
            requests: 1000,
            errors: 5,
            latency_ms: 23.4,
        };
        let map = m.into_hashmap();
        assert!(map.contains_key("requests"));
        assert!(map.contains_key("errors"));
        assert!(map.contains_key("latency_ms"));
        assert!(map["requests"].contains("1000"));
    }

    #[test]
    fn test_default_new() {
        let c = Counter::new();
        assert_eq!(c.value, 0);
        assert_eq!(c.name, "");
    }

    #[test]
    fn test_user_id_hash() {
        let mut map = HashMap::new();
        map.insert(UserId(1), "Alice");
        map.insert(UserId(2), "Bob");
        assert_eq!(map[&UserId(1)], "Alice");
    }

    #[test]
    fn test_score_ordering() {
        let s1 = Score::new(80.0, "Good");
        let s2 = Score::new(90.0, "Excellent");
        assert!(s1 < s2);
        assert!(s2 > s1);
        assert_eq!(s1, s1.clone());
    }

    #[test]
    fn test_cache_key_equality() {
        let k1 = CacheKey::new("users", "alice");
        let k2 = CacheKey::new("users", "alice");
        let k3 = CacheKey::new("users", "bob");

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);

        let mut set = std::collections::HashSet::new();
        set.insert(k1.clone());
        set.insert(k2.clone());
        assert_eq!(set.len(), 1); // 重複は1つ

        set.insert(k3);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_generic_wrapper() {
        let w1 = Wrapper { inner: 42_i32, metadata: "num".to_string() };
        let w2 = w1.clone();
        assert_eq!(w1, w2);

        let ws = Wrapper { inner: "hello".to_string(), metadata: "str".to_string() };
        assert_eq!(ws.inner, "hello");
    }
}
