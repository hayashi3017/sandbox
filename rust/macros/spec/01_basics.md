# macro_rules! の基礎

## 基本構文

```rust
macro_rules! マクロ名 {
    (パターン1) => { 展開1 };
    (パターン2) => { 展開2 };
    // ...
}
```

## 呼び出し形式

```rust
macro_name!()    // 丸括弧（慣習: 式として使う場合）
macro_name![]    // 角括弧（慣習: vec![], assert![]）
macro_name! {}   // 波括弧（慣習: 複数文を含む場合）
```

3つは完全に等価です。

## 主なフラグメント指定子（基礎）

| 指定子 | 用途 | 例 |
|---|---|---|
| `expr` | 式 | `1 + 2`, `"hello"` |
| `ident` | 識別子 | `foo`, `MyStruct` |
| `ty` | 型 | `i32`, `Vec<T>` |
| `literal` | リテラル | `42`, `"hi"` |

## 複数パターン（オーバーロード）

```rust
macro_rules! greet {
    () => { "Hello!" };
    ($name:expr) => { format!("Hello, {}!", $name) };
    ($greeting:expr, $name:expr) => { format!("{}, {}!", $greeting, $name) };
}
```

## 識別子からコード生成

```rust
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() -> &'static str {
            stringify!($func_name)
        }
    };
}
create_function!(hello); // fn hello() を生成
```

## 構造体の生成

```rust
macro_rules! define_struct {
    ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
        struct $name {
            $($field: $ty),+
        }
    };
}
define_struct!(Point { x: f64, y: f64 });
```

## 実装ファイル

`learning/src/basics.rs`
