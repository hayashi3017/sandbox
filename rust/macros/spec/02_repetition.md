# 繰り返し（Repetition）

## 繰り返し演算子

| 演算子 | 意味 | 類似 |
|---|---|---|
| `$(...)*` | 0回以上 | `*` |
| `$(...)+` | 1回以上 | `+` |
| `$(...)?` | 0または1回 | `?` |

## セパレータ

```text
$($x:expr),*   カンマ区切り（0以上）
$($x:expr);+   セミコロン区切り（1以上）
$($x:expr)+    区切りなし（1以上）
$(,)?          末尾カンマをオプションに
```

## 基本例

```rust
// vec!の実装に近い形
macro_rules! my_vec {
    ($($elem:expr),* $(,)?) => {
        vec![$($elem),*]
    };
}

// 複数値の合計
macro_rules! sum {
    ($($x:expr),*) => {{
        let mut total = 0;
        $(total += $x;)*
        total
    }};
}

assert_eq!(sum!(1, 2, 3, 4, 5), 15);
```

## HashMapリテラル

```rust
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

let m = hashmap! { "a" => 1, "b" => 2 };
```

## ネストされた繰り返し

```rust
macro_rules! matrix {
    ($([$($e:expr),+]),+) => {
        vec![$( vec![$($e),+] ),+]
    };
}

let m = matrix![[1, 2], [3, 4]];
```

## 引数の個数をカウント

```rust
macro_rules! count_args {
    ($($x:expr),*) => {{
        let arr: &[()] = &[$(replace_with_unit!($x)),*];
        arr.len()
    }};
}
```

## 実装ファイル

`learning/src/repetition.rs`
