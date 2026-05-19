# Deriveマクロ（Procedural Derive Macros）

## 概要

`#[derive(Name)]`でトレイト実装やメソッドを自動生成する手続きマクロです。

## 実装の仕組み

```
入力: 構造体/enumのTokenStream
       ↓ syn::parse_macro_input!でDeriveInputにパース
       ↓ フィールド情報を取り出す
       ↓ quote!でコード生成
出力: 追加するコードのTokenStream
```

### 最小限のderive実装

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl MyTrait for #name {
            fn hello(&self) -> &str { stringify!(#name) }
        }
    };
    TokenStream::from(expanded)
}
```

## このプロジェクトのDeriveマクロ

### `#[derive(Describe)]`

各フィールドの名前と値を文字列にする`describe()`メソッドを生成:

```rust
#[derive(Describe)]
struct Person {
    name: String,
    age: u32,
}

let p = Person { name: "Alice".to_string(), age: 30 };
let desc = p.describe();
// → "Person{ name: \"Alice\", age: 30 }"
assert!(desc.contains("name:"));
assert!(desc.contains("age:"));
```

**実装ポイント**: `Vec<String>`の型を明示しないとUnit structでコンパイルエラーになる。

```rust
quote! {
    let mut parts: Vec<String> = Vec::new();  // 型注釈が必要
    #(#field_descriptions)*
    format!("{}{{ {} }}", #struct_name, parts.join(", "))
}
```

### `#[derive(Builder)]`

Builderパターンを自動生成:

```rust
#[derive(Builder)]
struct Config {
    host: String,
    port: u16,
}

let c = Config::builder()
    .host("localhost".to_string())
    .port(8080)
    .build()
    .unwrap();

assert_eq!(c.host, "localhost");
assert_eq!(c.port, 8080);

// フィールドが不足するとErrが返る
let err = Config::builder().host("x".to_string()).build();
assert!(err.is_err());
```

**生成されるコード**:
- `ConfigBuilder` struct（全フィールドが`Option<T>`）
- `Config::builder() -> ConfigBuilder`
- 各フィールドのセッターメソッド（チェーン可能）
- `ConfigBuilder::build() -> Result<Config, String>`

### `#[derive(IntoHashMap)]`

構造体を`HashMap<String, String>`に変換する`into_hashmap()`を生成:

```rust
#[derive(IntoHashMap)]
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 1.0, y: 2.0 };
let map = p.into_hashmap();
assert!(map.contains_key("x"));
assert!(map.contains_key("y"));
```

### `#[derive(DefaultNew)]`

`Default`トレイトに基づく`new()`を生成:

```rust
#[derive(Default, DefaultNew)]
struct Counter {
    value: i32,
}

let c = Counter::new();  // Counter::default()と同じ
assert_eq!(c.value, 0);
```

## `syn`でのフィールド取り出し

```rust
let fields = match &input.data {
    Data::Struct(s) => &s.fields,
    _ => panic!("structにのみ適用可能"),
};

match fields {
    Fields::Named(named) => {
        for f in &named.named {
            let fname = f.ident.as_ref().unwrap();
            let ftype = &f.ty;
            // quote!でコード生成...
        }
    }
    Fields::Unnamed(unnamed) => {
        for (i, _) in unnamed.unnamed.iter().enumerate() {
            let idx = syn::Index::from(i);
            // self.0, self.1, ... でアクセス
        }
    }
    Fields::Unit => {}
}
```

## `format_ident!`でIdentifier生成

```rust
use quote::format_ident;

let name = &input.ident;             // 例: Config
let builder_name = format_ident!("{}Builder", name);  // ConfigBuilder
```

## Cargo.toml設定

```toml
# proc-macroクレートの設定
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1"
quote = "1"
syn = { version = "2", features = ["full"] }
```

**注意**: proc-macroクレートは単独のバイナリとして扱われる。使用側のクレートは別ワークスペースメンバーにする。

## 実装ファイル

- マクロ定義: `macros_impl/src/lib.rs`
- 使用例: `learning/src/proc_derive.rs`
