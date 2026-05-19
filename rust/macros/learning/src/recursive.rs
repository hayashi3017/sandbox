//! # 再帰マクロとTTマンチャー
//!
//! マクロは自分自身を再帰的に呼び出せます。
//! この性質を使ってリストの処理や複雑なDSLパーサーを実装できます。
//!
//! ## 再帰マクロの基本パターン
//!
//! ```text
//! macro_rules! my_macro {
//!     ()              => { /* ベースケース */ };
//!     ($head:tt $($tail:tt)*) => {
//!         /* headを処理し、tailを再帰 */
//!         my_macro!($($tail)*)
//!     };
//! }
//! ```
//!
//! ## TTマンチャー（Token Tree Muncher）
//!
//! 入力トークン列を先頭から「かじって」処理していくパターン。
//! 再帰の各ステップで1つ以上のトークンを消費し、残りを再帰的に処理します。
//!
//! ## Push-down Accumulation
//!
//! 結果を「蓄積引数」に積み上げていく再帰パターン。
//! Rustのマクロはleft-recursionが使えないため、このパターンが重要。

// ─── 基本的な再帰カウント ─────────────────────────────────────────────────────

/// トークンの個数をコンパイル時に数える
///
/// 空リスト→0、非空→1+残りの再帰、というベースケース+再帰の構造。
macro_rules! count_tts {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => {
        1usize + count_tts!($($tail)*)
    };
}

/// カンマ区切りの引数の個数を数える（より実用的な版）
macro_rules! count_exprs {
    () => { 0usize };
    ($head:expr $(, $tail:expr)*) => {
        1usize + count_exprs!($($tail),*)
    };
}

// ─── リスト処理 ─────────────────────────────────────────────────────────────

/// 先頭要素を取り出す
macro_rules! head {
    ($head:expr $(, $_tail:expr)*) => { $head };
}

/// 末尾要素を取り出す
macro_rules! tail {
    ($only:expr) => { $only };
    ($_head:expr, $($rest:expr),+) => { tail!($($rest),+) };
}

/// Nthの要素を取り出す（インデックスゼロ始まり）
macro_rules! nth {
    (0, $first:expr $(, $_rest:expr)*) => { $first };
    ($n:expr, $_first:expr $(, $rest:expr)+) => {
        nth!($n - 1, $($rest),+)
    };
}

/// リストを逆順にする（Push-down Accumulation）
///
/// `@rev`の内部ルールで公開インターフェースと再帰処理を分離する。
/// 分離しないと`[$(...)]`が外部呼び出しのexprとしてマッチし無限再帰になる。
macro_rules! reverse_list {
    // 内部ルール: ベースケース（リストが空になった）
    (@rev [$($acc:expr),*]) => {
        [$($acc),*]
    };
    // 内部ルール: 再帰（先頭を蓄積の先頭に積む）
    (@rev [$($acc:expr),*] $head:expr $(, $tail:expr)*) => {
        reverse_list!(@rev [$head $(, $acc)*] $($tail),*)
    };
    // 公開インターフェース: 内部ルールに委譲
    ($($xs:expr),*) => {
        reverse_list!(@rev [] $($xs),*)
    };
}

// ─── 算術の再帰 ─────────────────────────────────────────────────────────────

/// 再帰で合計を計算するマクロ
macro_rules! sum_recursive {
    () => { 0 };
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        $x + sum_recursive!($($rest),+)
    };
}

/// 再帰で最大値を求めるマクロ
macro_rules! max_recursive {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let rest = max_recursive!($($rest),+);
        if $x > rest { $x } else { rest }
    }};
}

// ─── TTマンチャー ────────────────────────────────────────────────────────────

/// カスタム演算子`+`と`*`を処理するTTマンチャー
///
/// `calc!(1 + 2 * 3)` → 計算を左から右に評価する（優先度なし）
macro_rules! calc {
    // ベースケース: 数値のみ
    ($x:literal) => { $x };
    // 加算
    ($x:literal + $($rest:tt)+) => {
        $x + calc!($($rest)+)
    };
    // 乗算
    ($x:literal * $($rest:tt)+) => {
        $x * calc!($($rest)+)
    };
    // 減算
    ($x:literal - $($rest:tt)+) => {
        $x - calc!($($rest)+)
    };
}

/// カスタムDSL: 簡易フィルタリング言語
///
/// `filter!((v) WHERE x > 0)` → vecのフィルタリング
/// exprは後続が限られるため、対象をカッコで包む
macro_rules! filter {
    (($vec:expr) WHERE $var:ident $op:tt $val:expr) => {
        $vec.iter().filter(|$var| *$var $op &$val).copied().collect::<Vec<_>>()
    };
}

// ─── Push-down Accumulation パターン ────────────────────────────────────────

/// フィボナッチ数列の最初のN項をコンパイル時に生成する
///
/// Push-down accumulationで数列を蓄積していく。
macro_rules! fib_seq {
    // 公開インターフェース: N項を要求
    ($n:literal) => {
        fib_seq!(@acc [$n] [0, 1])
    };
    // 残り0: 蓄積を返す
    (@acc [0] [$($acc:expr),+]) => {
        [$($acc),+]
    };
    // 残り1: 最後の蓄積をそのまま返す
    (@acc [1] [$($acc:expr),+]) => {
        [$($acc),+]
    };
    // 再帰: 次の値を蓄積に追加
    // （この簡易版は固定の展開を使う）
    (@acc [$n:literal] [$($acc:expr),+]) => {
        // 実用的なfibは型システムの限界から複雑になるため、
        // ここでは有限の例を示す
        compile_error!("fib_seq supports up to 2 steps in this demo")
    };
}

/// 識別子のリストをstringifyして文字列ベクタにする
///
/// `parse_words!(hello world foo)` → vec!["hello", "world", "foo"]
macro_rules! parse_words {
    ($($word:ident)*) => {
        vec![$(stringify!($word)),*]
    };
}

/// 型の「スタック」を再帰的に積むマクロ（push-down）
macro_rules! type_stack {
    // 公開インターフェース: Tで初期化
    ($t:ty) => {
        type_stack!(@build [] $t)
    };
    (@build [$($acc:ty),*] $t:ty) => {
        ($t, ($($acc,)* ))
    };
}

// ─── 相互再帰マクロ ────────────────────────────────────────────────────────

/// 偶数判定マクロ（相互再帰のデモ）
///
/// **注意**: マクロはトークンを操作するため、`2 - 1`は`1`に評価されない。
/// そのため、0と1のリテラルのみが正しく動作する。
/// 実際の数値の偶奇判定にはこの方法は使えない。
macro_rules! is_even {
    (0) => { true };
    (1) => { false };
}

macro_rules! is_odd {
    (0) => { false };
    (1) => { true };
}

// ─── マクロによるコード生成 ──────────────────────────────────────────────────

/// テストケースを自動生成するマクロ
macro_rules! test_cases {
    ($func:ident: $($input:expr => $expected:expr),+ $(,)?) => {
        $(
            {
                let result = $func($input);
                assert_eq!(
                    result,
                    $expected,
                    "{}({:?}) expected {:?}, got {:?}",
                    stringify!($func),
                    $input,
                    $expected,
                    result
                );
            }
        )+
    };
}

fn double_fn(x: i32) -> i32 {
    x * 2
}
fn square_fn(x: i32) -> i32 {
    x * x
}

// ─── Callback パターン ────────────────────────────────────────────────────────

/// コールバックマクロを受け取るマクロ
///
/// マクロを別のマクロに渡すパターン（高階マクロ）
macro_rules! apply_to_each {
    ($callback:ident![$($items:expr),+]) => {
        vec![$($callback!($items)),+]
    };
}

macro_rules! stringify_item {
    ($x:expr) => {
        format!("{:?}", $x)
    };
}

macro_rules! square_item {
    ($x:expr) => {
        $x * $x
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tts() {
        assert_eq!(count_tts!(), 0);
        assert_eq!(count_tts!(a), 1);
        assert_eq!(count_tts!(a b c), 3);
        assert_eq!(count_tts!(1 2 3 4 5), 5);
    }

    #[test]
    fn test_count_exprs() {
        assert_eq!(count_exprs!(), 0);
        assert_eq!(count_exprs!(1), 1);
        assert_eq!(count_exprs!(1, 2, 3), 3);
    }

    #[test]
    fn test_head() {
        assert_eq!(head!(10, 20, 30), 10);
        assert_eq!(head!("first", "second"), "first");
    }

    #[test]
    fn test_tail() {
        assert_eq!(tail!(10), 10);
        assert_eq!(tail!(10, 20, 30), 30);
    }

    #[test]
    fn test_reverse_list() {
        let r = reverse_list![1, 2, 3, 4, 5];
        assert_eq!(r, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_sum_recursive() {
        assert_eq!(sum_recursive!(), 0);
        assert_eq!(sum_recursive!(5), 5);
        assert_eq!(sum_recursive!(1, 2, 3, 4, 5), 15);
    }

    #[test]
    fn test_max_recursive() {
        assert_eq!(max_recursive!(3), 3);
        assert_eq!(max_recursive!(1, 5, 3, 2, 4), 5);
        assert_eq!(max_recursive!(-10, -5, -1), -1);
    }

    #[test]
    fn test_calc() {
        assert_eq!(calc!(5), 5);
        assert_eq!(calc!(2 + 3), 5);
        assert_eq!(calc!(3 * 4), 12);
        assert_eq!(calc!(10 - 3), 7);
        assert_eq!(calc!(1 + 2 + 3), 6);
    }

    #[test]
    fn test_filter() {
        let v = vec![1, -2, 3, -4, 5];
        let positive: Vec<i32> = filter!((v) WHERE x > 0);
        assert_eq!(positive, vec![1, 3, 5]);

        let large: Vec<i32> = filter!((v) WHERE x > 2);
        assert_eq!(large, vec![3, 5]);
    }

    #[test]
    fn test_parse_words() {
        let words = parse_words!(hello world foo);
        assert_eq!(words, vec!["hello", "world", "foo"]);

        let empty: Vec<&str> = parse_words!();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_test_cases_macro() {
        test_cases! {
            double_fn:
            0  => 0,
            1  => 2,
            5  => 10,
            -3 => -6,
        }

        test_cases! {
            square_fn:
            0 => 0,
            3 => 9,
            5 => 25,
        }
    }

    #[test]
    fn test_apply_to_each() {
        let strings = apply_to_each!(stringify_item![1, 2, 3]);
        assert_eq!(strings, vec!["1", "2", "3"]);

        let squares = apply_to_each!(square_item![2, 3, 4]);
        assert_eq!(squares, vec![4, 9, 16]);
    }

    #[test]
    fn test_is_even_odd() {
        // マクロはトークン評価のためリテラル0と1のみ動作する
        assert!(is_even!(0));
        assert!(!is_even!(1));
        assert!(!is_odd!(0));
        assert!(is_odd!(1));
    }
}
