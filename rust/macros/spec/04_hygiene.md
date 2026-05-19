# マクロ衛生性（Macro Hygiene）

## 概要

Rustの`macro_rules!`は**衛生的（hygienic）**です。
マクロ内で導入した識別子は呼び出し元のスコープと干渉しません。

## 衛生性のデモ

```rust
macro_rules! with_temp {
    ($e:expr) => {{
        let val = $e;  // このvalは呼び出し元のvalと別物
        val * 2
    }};
}

let val = 100;           // 呼び出し元のval
let result = with_temp!(5);  // マクロ内のval=5
assert_eq!(val, 100);    // 呼び出し元のvalは変わらない
assert_eq!(result, 10);  // 5 * 2 = 10
```

## 意図的な識別子の注入

`ident`フラグメントで変数名を受け取ると、呼び出し元のスコープに束縛できる:

```rust
macro_rules! for_range {
    ($var:ident in $range:expr => $body:block) => {
        for $var in $range $body  // $varは呼び出し元が命名
    };
}

for_range!(i in 0..5 => { println!("{}", i); });
```

## `$crate`

`#[macro_export]`されたマクロが他クレートで使われる際に、
定義元クレートの項目を確実に参照するための特殊変数:

```rust
#[macro_export]
macro_rules! make_error {
    ($msg:literal) => {
        // $crateがなければ、他クレートでMyErrorが見つからない
        $crate::MyError::new($msg)
    };
}
```

## `stringify!` と `concat!`

```rust
// stringify!: トークンをそのまま文字列化
macro_rules! debug_expr {
    ($e:expr) => {{
        let result = $e;
        println!("{} = {:?}", stringify!($e), result);
    }};
}

debug_expr!(1 + 2 * 3);  // "1 + 2 * 3 = 7"

// concat!: コンパイル時の文字列結合
const MSG: &str = concat!("Error", ": ", "not found");
```

## 実装ファイル

`learning/src/hygiene.rs`
