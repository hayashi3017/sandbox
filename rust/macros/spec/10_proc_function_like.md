# 関数形式マクロ（Procedural Function-like Macros）

## 概要

`name!(...)` 形式の手続きマクロです。`macro_rules!`と同じ呼び出し形式ですが、Rustコードではなくプログラムでトークンを処理できます。

## 実装の仕組み

```rust
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // input: マクロに渡されたトークン列
    // 戻り値: 展開後のコード
}
```

`macro_rules!`との違い: パターンマッチングではなくRustコードで任意の処理ができる。複雑なDSLパーサーや外部リソース連携に適している。

## このプロジェクトの関数形式マクロ

### `count_tokens!(...)`

トークンの個数をコンパイル時にカウント:

```rust
use macros_impl::count_tokens;

assert_eq!(count_tokens!(), 0);
assert_eq!(count_tokens!(a b c), 3);
assert_eq!(count_tokens!(1 + 2), 3);  // 1, +, 2 の3トークン
assert_eq!(count_tokens!(a, b, c), 5);  // a, ,, b, ,, c の5トークン
```

**実装**: `TokenStream::into_iter().count()`で単純カウント:

```rust
#[proc_macro]
pub fn count_tokens(input: TokenStream) -> TokenStream {
    let count = input.into_iter().count();
    let expanded = quote! { #count };
    TokenStream::from(expanded)
}
```

### `sql!(...)`

簡易SQLパーサー（`SELECT ... FROM ...`を構造体に変換）:

```rust
use macros_impl::sql;

let q = sql!(SELECT name, age FROM users);
assert_eq!(q.table, "users");
assert_eq!(q.columns, vec!["name", "age"]);

let q2 = sql!(SELECT id, title, content FROM posts);
assert_eq!(q2.table, "posts");
assert_eq!(q2.columns.len(), 3);
```

**生成されるコード**（概念）:

```rust
{
    struct SqlQuery {
        pub table: &'static str,
        pub columns: Vec<&'static str>,
    }
    SqlQuery {
        table: "users",
        columns: vec!["name", "age"],
    }
}
```

**実装**: トークンをイテレートしてステートマシンでパース:

```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut columns = Vec::new();
    let mut table = String::new();
    let mut mode = "start";

    for token in &tokens {
        let s = token.to_string();
        match (mode, s.as_str()) {
            ("start", "SELECT")  => mode = "columns",
            ("columns", "FROM")  => mode = "table",
            ("columns", ",")     => {}
            ("columns", col)     => columns.push(col.to_string()),
            ("table", tbl)       => { table = tbl.to_string(); mode = "done"; }
            _ => {}
        }
    }
    // quote!でSqlQuery構造体を生成...
}
```

### `make_enum!(...)`

enumと文字列変換メソッドを生成:

```rust
use macros_impl::make_enum;

make_enum!(Color { Red, Green, Blue });

assert_eq!(Color::Red.as_str(), "Red");
assert_eq!(format!("{}", Color::Green), "Green");
assert_eq!(Color::Blue, Color::Blue);
```

**生成されるコード**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color { Red, Green, Blue }

impl Color {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Red   => "Red",
            Self::Green => "Green",
            Self::Blue  => "Blue",
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

**実装**: `proc_macro2::TokenTree::Group`でブレース内のidentを取り出す:

```rust
let variants: Vec<String> = tokens.iter()
    .find_map(|t| {
        if let proc_macro2::TokenTree::Group(g) = t {
            Some(g.stream().into_iter()
                .filter_map(|t| {
                    if let proc_macro2::TokenTree::Ident(id) = t {
                        Some(id.to_string())
                    } else { None }
                })
                .collect())
        } else { None }
    })
    .unwrap_or_default();
```

## `proc_macro2` と `proc_macro` の使い分け

| | `proc_macro` | `proc_macro2` |
|---|---|---|
| 使用場所 | proc-macroクレートのみ | 通常のクレートでも使える |
| テスト | 困難 | 容易（通常のユニットテスト可） |
| 変換 | `TokenStream::from(ts2)` | `TokenStream2::from(ts)` |

```rust
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();   // proc_macro → proc_macro2
    // proc_macro2で処理...
    let result: TokenStream2 = quote! { ... };
    TokenStream::from(result)                   // proc_macro2 → proc_macro
}
```

## `macro_rules!`との使い分け

| 要件 | 推奨 |
|---|---|
| 単純なパターンマッチング | `macro_rules!` |
| 複雑な構文解析 | 手続きマクロ |
| コンパイル時の外部リソース読み込み | 手続きマクロ |
| ファイル生成・変換 | 手続きマクロ |
| 可読性・保守性重視 | `macro_rules!` |

## 実装ファイル

- マクロ定義: `macros_impl/src/lib.rs`
- 使用例: `learning/src/proc_function_like.rs`
