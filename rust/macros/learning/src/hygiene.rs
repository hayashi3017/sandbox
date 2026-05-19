//! # マクロ衛生性（Macro Hygiene）
//!
//! Rustの`macro_rules!`マクロは**衛生的（hygienic）**です。
//! マクロ内で導入された変数や識別子は、呼び出し元のスコープと干渉しません。
//!
//! ## 衛生性の効果
//!
//! 1. **識別子の独立性**: マクロ内の`x`と呼び出し元の`x`は別物
//! 2. **シャドーイング防止**: マクロが偶発的に変数を隠さない
//! 3. **安全なコード生成**: マクロが既存の変数を壊さない
//!
//! ## `$crate`
//!
//! マクロ定義クレートの項目を確実に参照するための特殊変数。
//! `#[macro_export]`されたマクロが他クレートで使われる際に、
//! パスを正しく解決するために使います。
//!
//! ## 衛生性の限界
//!
//! 呼び出し元の識別子を`ident`フラグメントで受け取ると、
//! 呼び出し元のスコープに「意図的に」アクセスできます。
//! これを使って「非衛生的」なマクロも作れますが、慎重に使う必要があります。

// ─── 基本的な衛生性のデモ ─────────────────────────────────────────────────────

/// 衛生的なマクロの例
///
/// マクロ内の`inner`変数は呼び出し元の同名変数と干渉しない。
macro_rules! swap {
    ($a:expr, $b:expr) => {{
        // このtmpは呼び出し元のtmpとは別の変数（衛生的）
        let mut tmp = $a;
        $a = $b;
        $b = tmp;
    }};
}

pub fn demo_hygiene_swap() -> (i32, i32) {
    let mut x = 1;
    let mut y = 2;
    let tmp = 999; // この tmp はswap!内のtmpと干渉しない
    swap!(x, y);
    (x, y + tmp * 0) // tmp は999のまま
}

/// 衛生性のデモ: マクロ内の変数名が外部と競合しない
macro_rules! with_temp {
    ($e:expr) => {{
        // このvalは呼び出し元のvalとは別
        let val = $e;
        val * 2
    }};
}

pub fn hygiene_no_conflict() -> (i32, i32) {
    let val = 100; // 呼び出し元のval
    let result = with_temp!(5); // マクロ内のval=5、外のval=100は変わらない
    (val, result) // (100, 10)
}

// ─── 識別子の故意的な注入（非衛生的パターン） ────────────────────────────────

/// `ident`を受け取って呼び出し元のスコープにバインドするマクロ
///
/// `$var:ident`でユーザーが変数名を指定するため、
/// 呼び出し元のスコープに意図的にバインドできる。
macro_rules! let_bind {
    (let $var:ident = $val:expr) => {
        let $var = $val;
    };
}

pub fn intentional_binding() -> i32 {
    let_bind!(let answer = 42);
    answer // answerはマクロによって導入されたが、呼び出し元が名前を指定した
}

/// ループ変数の名前を呼び出し元が指定するマクロ
macro_rules! for_range {
    ($var:ident in $range:expr => $body:block) => {
        for $var in $range $body
    };
}

pub fn named_loop() -> Vec<i32> {
    let mut results = Vec::new();
    for_range!(i in 0..5 => {
        results.push(i * i);
    });
    results
}

// ─── $crate の使用 ───────────────────────────────────────────────────────────

/// `$crate`を使ったマクロ
///
/// このマクロが他クレートから`#[macro_use]`や`use`で使われたとき、
/// `$crate::SomeType`は定義元クレートの型を正しく参照する。
///
/// 以下はクレート内での使用例（他クレートからの使用は実際の公開時に有効）。
#[macro_export]
macro_rules! make_error {
    ($msg:literal) => {
        // $crateを使うことで、このマクロが他クレートで使われても
        // MyErrorは正しく参照される
        $crate::hygiene::MyError::new($msg)
    };
}

#[derive(Debug, PartialEq)]
pub struct MyError {
    pub message: String,
}

impl MyError {
    pub fn new(msg: &str) -> Self {
        MyError {
            message: msg.to_string(),
        }
    }
}

// ─── stringify! と concat! ──────────────────────────────────────────────────

/// `stringify!`はトークンをそのまま文字列化する
macro_rules! debug_expr {
    ($e:expr) => {{
        let result = $e;
        (stringify!($e), result)
    }};
}

/// `concat!`はコンパイル時に文字列を結合する
macro_rules! make_message {
    ($prefix:literal, $($part:literal),+) => {
        concat!($prefix, ": ", $($part),+)
    };
}

// ─── 型エイリアスとパスの衛生性 ────────────────────────────────────────────

/// 型を衛生的に参照するマクロ
macro_rules! make_hash_map {
    ($key:ty, $val:ty) => {
        // std::collections::HashMapを完全パスで参照
        std::collections::HashMap::<$key, $val>::new()
    };
}

// ─── マクロの衛生性とスコープ ────────────────────────────────────────────────

/// スコープが限定されたマクロの例
///
/// マクロはテキスト展開ではなく、構文的な展開を行う。
/// そのためスコープルールは通常のRustコードと同様に適用される。
macro_rules! scoped_val {
    ($val:expr) => {{
        // このブロックスコープ内の変数は外に漏れない
        let internal = $val;
        internal + 1
    }};
}

pub fn scope_demo() -> i32 {
    let result = scoped_val!(41);
    // `internal`はここでは参照できない（スコープ外）
    result
}

// ─── 衛生的な内部状態マクロ ─────────────────────────────────────────────────

/// マクロが内部で複数の一時変数を使うが衛生的に保つ例
macro_rules! benchmark {
    ($e:expr) => {{
        let _start = std::time::Instant::now();
        let _result = $e;
        let _end = std::time::Instant::now();
        (_result, _end.duration_since(_start))
    }};
}

pub fn benchmark_demo() -> i32 {
    let (result, _duration) = benchmark!(1 + 2 + 3);
    result
}

// ─── 衛生性のまとめデモ関数 ─────────────────────────────────────────────────

pub fn show_all_hygiene() -> Vec<(&'static str, bool)> {
    vec![
        ("swap: x,y交換後 (x=2, y=1)", demo_hygiene_swap() == (2, 1)),
        ("no_conflict: val=100, result=10", hygiene_no_conflict() == (100, 10)),
        ("binding: answer=42", intentional_binding() == 42),
        ("scope: scoped_val(41)=42", scope_demo() == 42),
        ("benchmark: 1+2+3=6", benchmark_demo() == 6),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_hygiene() {
        // tmpという変数が呼び出し元にあっても干渉しない
        let tmp = 999_i32;
        let mut a = 10_i32;
        let mut b = 20_i32;
        swap!(a, b);
        assert_eq!(a, 20);
        assert_eq!(b, 10);
        assert_eq!(tmp, 999); // tmpは変わらない
    }

    #[test]
    fn test_with_temp_no_conflict() {
        let val = 100; // 呼び出し元のval
        let result = with_temp!(5); // マクロ内のvalは5
        assert_eq!(val, 100); // 呼び出し元のvalは変わらない
        assert_eq!(result, 10); // 5 * 2 = 10
    }

    #[test]
    fn test_hygiene_no_conflict() {
        let (outer_val, result) = hygiene_no_conflict();
        assert_eq!(outer_val, 100);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_let_bind() {
        let_bind!(let x = 42);
        assert_eq!(x, 42);

        let_bind!(let greeting = "hello");
        assert_eq!(greeting, "hello");
    }

    #[test]
    fn test_for_range() {
        let mut sum = 0;
        for_range!(n in 1..=10 => {
            sum += n;
        });
        assert_eq!(sum, 55);
    }

    #[test]
    fn test_make_error() {
        let e = make_error!("something went wrong");
        assert_eq!(e.message, "something went wrong");
    }

    #[test]
    fn test_debug_expr() {
        let (expr_str, value) = debug_expr!(1 + 2 * 3);
        assert_eq!(expr_str, "1 + 2 * 3");
        assert_eq!(value, 7);
    }

    #[test]
    fn test_make_message() {
        const MSG: &str = make_message!("Error", "file not found");
        assert_eq!(MSG, "Error: file not found");
    }

    #[test]
    fn test_make_hash_map() {
        let mut m = make_hash_map!(String, i32);
        m.insert("key".to_string(), 42);
        assert_eq!(m["key"], 42);
    }

    #[test]
    fn test_scope_demo() {
        assert_eq!(scope_demo(), 42);
    }

    #[test]
    fn test_named_loop() {
        let squares = named_loop();
        assert_eq!(squares, vec![0, 1, 4, 9, 16]);
    }

    #[test]
    fn test_show_all_hygiene() {
        let results = show_all_hygiene();
        for (name, ok) in &results {
            assert!(ok, "Failed: {}", name);
        }
    }

    #[test]
    fn test_benchmark() {
        let (result, duration) = benchmark!(vec![1, 2, 3].iter().sum::<i32>());
        assert_eq!(result, 6);
        // durationはゼロより大きい（ナノ秒単位）
        assert!(duration.as_nanos() >= 0);
    }
}
