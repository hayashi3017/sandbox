# 高度なマクロパターン（Advanced Macro Patterns）

## 概要

実用的なマクロ設計で使われる高度なパターン集です。

## Internal Rules（`@`タグパターン）

`@`で始まるアームを「内部ルール」として使い、公開インターフェースと実装を分離する慣習。

```rust
macro_rules! my_format {
    // 公開インターフェース
    ($fmt:literal $(, $arg:expr)*) => {
        my_format!(@build [] $fmt $(, $arg)*)
    };
    // 内部ルール: 引数なし
    (@build [$($parts:expr),*] $fmt:literal) => {
        format!($fmt)
    };
    // 内部ルール: 引数あり
    (@build [$($parts:expr),*] $fmt:literal, $first:expr $(, $rest:expr)*) => {
        format!($fmt, $first $(, $rest)*)
    };
}

assert_eq!(my_format!("Hello, {}!", "world"), "Hello, world!");
```

## エラー処理パターン

```rust
macro_rules! try_all {
    ($e:expr) => { $e? };
    ($first:expr, $($rest:expr),+) => {{
        $first?;
        try_all!($($rest),+)
    }};
}

fn validate_all(a: i32, b: i32, c: i32) -> Result<i32, String> {
    fn check_positive(x: i32) -> Result<(), String> {
        if x > 0 { Ok(()) } else { Err(format!("{} is not positive", x)) }
    }
    try_all!(check_positive(a), check_positive(b), check_positive(c));
    Ok(a + b + c)
}
```

## 状態機械（State Machine）DSL

状態とイベントを分離して列挙し、遷移リストで参照する構造。重複したvariantの生成を防ぐ。

```rust
macro_rules! state_machine {
    (
        states: [$($state:ident),+ $(,)?],
        events: [$($event:ident),+ $(,)?],
        initial: $initial:ident,
        transitions: [
            $($from:ident --[$ev:ident]--> $to:ident),+ $(,)?
        ]
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum State { $($state),+ }

        #[derive(Debug, Clone, PartialEq)]
        pub enum Event { $($event),+ }

        pub struct Machine { pub state: State }

        impl Machine {
            pub fn new() -> Self { Machine { state: State::$initial } }

            pub fn transition(&mut self, event: &Event) -> bool {
                let next = match (&self.state, event) {
                    $(
                        (State::$from, Event::$ev) => Some(State::$to),
                    )+
                    _ => None,
                };
                if let Some(s) = next { self.state = s; true } else { false }
            }
        }
    };
}

state_machine! {
    states: [Idle, Running, Paused, Stopped],
    events: [Start, Pause, Resume, Stop],
    initial: Idle,
    transitions: [
        Idle    --[Start]-->  Running,
        Running --[Pause]-->  Paused,
        Paused  --[Resume]--> Running,
        Running --[Stop]-->   Stopped,
        Paused  --[Stop]-->   Stopped,
    ]
}
```

**設計のポイント**: `events: [...]`を別に宣言することで、`Stop`のような複数遷移で使われるイベントが`Event` enumに一度だけ現れる。`transitions:`だけから生成しようとすると重複variantになる。

## トークン分類と演算子切り替え

```rust
macro_rules! is_plus {
    (+) => { true };
    ($other:tt) => { false };
}

macro_rules! apply_op {
    ($a:expr, +, $b:expr) => { $a + $b };
    ($a:expr, -, $b:expr) => { $a - $b };
    ($a:expr, *, $b:expr) => { $a * $b };
    ($a:expr, /, $b:expr) => { $a / $b };
}

assert_eq!(apply_op!(10, +, 5), 15);
assert_eq!(apply_op!(10, *, 5), 50);
```

## ドキュメントコメントの自動生成

`#[doc = ...]`はマクロで展開できるため、動的なドキュメントを生成できる。

```rust
macro_rules! documented_const {
    ($name:ident: $ty:ty = $val:expr, doc: $doc:literal) => {
        #[doc = $doc]
        pub const $name: $ty = $val;
    };
}

documented_const!(MAX_RETRIES: u32 = 3, doc: "最大リトライ回数");
documented_const!(TIMEOUT_MS: u64 = 5000, doc: "タイムアウト（ミリ秒）");
```

## 条件コンパイルの抽象化

```rust
macro_rules! platform_impl {
    (
        #[cfg($condition:meta)]
        $item:item
        otherwise: $fallback:item
    ) => {
        #[cfg($condition)]
        $item

        #[cfg(not($condition))]
        $fallback
    };
}

platform_impl! {
    #[cfg(target_pointer_width = "64")]
    pub fn pointer_size() -> usize { 8 }
    otherwise:
    pub fn pointer_size() -> usize { 4 }
}
```

## 配列初期化パターン

```rust
macro_rules! array_init {
    ($val:expr; $n:literal) => {
        [$val; $n]
    };
    ($($val:expr),+ $(,)?) => {
        [$($val),+]
    };
}

let zeros = array_init!(0; 5);    // [0, 0, 0, 0, 0]
let nums  = array_init!(1, 2, 3); // [1, 2, 3]
```

## テーブル駆動テストパターン

```rust
macro_rules! table_tests {
    (fn $name:ident($($param:ident : $pty:ty),+) -> $ret:ty {
        $($input:expr => $expected:expr),+ $(,)?
    } with $func:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;
            #[test]
            fn run_all() {
                let f = $func;
                $({
                    let result = f($input);
                    assert_eq!(result, $expected);
                })+
            }
        }
    };
}
```

## 末尾カンマ対応の慣用パターン

```rust
// $(,)? で末尾カンマをオプションにする
macro_rules! my_macro {
    ($($item:expr),+ $(,)?) => { /* ... */ };
}

my_macro!(1, 2, 3);   // OK
my_macro!(1, 2, 3,);  // 末尾カンマもOK
```

## 実装ファイル

`learning/src/patterns.rs`
