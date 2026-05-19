# フラグメント指定子（Fragment Specifiers）

## 全フラグメント指定子

| 指定子 | 受け入れる構文 | 例 |
|---|---|---|
| `expr` | 式 | `1+2`, `foo()`, `"hi"` |
| `ident` | 識別子 | `foo`, `MyType` |
| `ty` | 型 | `i32`, `Vec<T>`, `&'a str` |
| `literal` | リテラルのみ | `42`, `"hello"`, `true` |
| `block` | ブロック式 | `{ let x=1; x }` |
| `stmt` | 文 | `let x = 1;` |
| `pat` | パターン | `Some(x)`, `(a, b)` |
| `pat_param` | パターン（`\|`後続可） | `1 \| 2 \| 3` |
| `item` | アイテム | `fn foo(){}`, `struct Bar` |
| `meta` | 属性メタデータ | `derive(Debug)` |
| `tt` | トークンツリー（任意） | あらゆるトークン |
| `lifetime` | ライフタイム | `'a`, `'static` |
| `vis` | 可視性修飾子 | `pub`, `pub(crate)` |
| `path` | パス | `std::collections::HashMap` |

## 後続制限（Follow Set）

特定フラグメントの後に続けられるトークンに制限があります:

| フラグメント | 後続可能なトークン |
|---|---|
| `expr`, `stmt` | `=>`, `,`, `;` |
| `ty`, `path` | `=>`, `,`, `>`, `[`, `{`, `as`, `where`, ブロック区切り |
| `pat`, `pat_param` | `=>`, `,`, `=`, `\|`, `if`, `in` |
| `vis` | `,`、識別子、`priv`、`pub`、型の開始 |
| `tt`, `ident`, `lifetime`, `block`, `item`, `meta`, `literal` | 制限なし |

## 各フラグメントの使用例

### `tt`（最汎用）

```rust
// 任意のトークンを受け入れる
macro_rules! identity {
    ($($t:tt)*) => { $($t)* };
}
identity!(let x = 1 + 2; x * 3)
```

### `ident`（識別子生成）

```rust
macro_rules! make_newtype {
    ($vis:vis $name:ident($inner:ty)) => {
        $vis struct $name($inner);
        impl $name {
            $vis fn new(val: $inner) -> Self { $name(val) }
        }
    };
}
make_newtype!(pub UserId(u64));
```

### `meta`（属性メタデータ）

```rust
macro_rules! with_attr {
    (#[$m:meta] $item:item) => {
        #[$m]
        $item
    };
}
with_attr! {
    #[derive(Debug, Clone)]
    struct MyStruct { value: i32 }
}
```

## 実装ファイル

`learning/src/fragments.rs`
