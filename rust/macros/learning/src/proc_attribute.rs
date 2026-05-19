//! # 属性マクロ（Attribute Macros）
//!
//! 属性マクロは`#[attr_name]`の形式でアイテム（関数・構造体等）を変換します。
//! Deriveマクロと異なり、アイテム全体を書き換えることができます。
//!
//! ## 特徴
//!
//! - アイテムの**完全な書き換え**が可能（Deriveはコードを追加するだけ）
//! - 関数・構造体・モジュール等に適用できる
//! - 引数を取ることができる（`#[retry(times = 3)]`）
//!
//! ## このモジュールのカスタム属性マクロ
//!
//! | マクロ | 効果 |
//! |---|---|
//! | `#[logged]` | 関数の入口/出口をログ出力 |
//! | `#[retry(times = N)]` | 失敗時にN回リトライ |
//! | `#[measure_time]` | 実行時間を計測 |

use macros_impl::{logged, measure_time, retry};

// ─── #[logged] ───────────────────────────────────────────────────────────────

/// ログ出力が追加された加算関数
#[logged]
pub fn add_logged(a: i32, b: i32) -> i32 {
    a + b
}

/// ログ出力が追加された文字列結合関数
#[logged]
pub fn concat_logged(a: &str, b: &str) -> String {
    format!("{}{}", a, b)
}

// ─── #[retry(times = N)] ──────────────────────────────────────────────────────

/// 成功するまでリトライするシミュレーション
///
/// 静的カウンターで「N回目で成功」をシミュレートする。
static ATTEMPT_COUNT: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);

pub fn reset_attempt_count() {
    ATTEMPT_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

/// 3回目の試行で成功する関数（retryのテスト用）
#[retry(times = 3)]
pub fn succeed_on_third_try() -> Result<i32, String> {
    let n = ATTEMPT_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if n < 2 {
        Err(format!("attempt {} failed", n + 1))
    } else {
        Ok(n + 1)
    }
}

/// 常に成功する関数（リトライ不要）
#[retry(times = 3)]
pub fn always_succeeds() -> Result<i32, String> {
    Ok(42)
}

/// 常に失敗する関数（全リトライ後にErr）
#[retry(times = 3)]
pub fn always_fails() -> Result<i32, String> {
    Err("always fails".to_string())
}

// ─── #[measure_time] ──────────────────────────────────────────────────────────

/// 実行時間計測付きのソート関数
#[measure_time]
pub fn sort_numbers(mut nums: Vec<i32>) -> Vec<i32> {
    nums.sort();
    nums
}

/// 実行時間計測付きの合計計算
#[measure_time]
pub fn sum_range(n: u64) -> u64 {
    (1..=n).sum()
}

// ─── 属性マクロを使った実用的なパターン ────────────────────────────────────

/// 複数の属性マクロを組み合わせる
#[measure_time]
#[logged]
pub fn complex_operation(data: Vec<i32>) -> Vec<i32> {
    data.iter().map(|x| x * x).filter(|x| *x > 10).collect()
}

// ─── 条件コンパイルと属性 ────────────────────────────────────────────────────

/// `#[cfg_attr]`で条件付き属性を付与する
///
/// これは標準機能（proc_macroではない）の例だが、属性マクロと一緒に理解する。
#[cfg_attr(debug_assertions, logged)]
pub fn conditionally_logged(x: i32) -> i32 {
    x * 2
}

// ─── 標準的な属性との連携 ────────────────────────────────────────────────────

/// `#[allow]`、`#[must_use]`等の組み込み属性との共存
#[allow(dead_code)]
#[must_use]
#[logged]
pub fn important_operation(x: i32) -> i32 {
    x + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logged_add() {
        // #[logged]はprintlnを追加するが、返り値は変わらない
        assert_eq!(add_logged(3, 4), 7);
        assert_eq!(add_logged(-1, 1), 0);
    }

    #[test]
    fn test_logged_concat() {
        assert_eq!(concat_logged("hello", " world"), "hello world");
    }

    #[test]
    fn test_retry_success_eventually() {
        reset_attempt_count();
        let result = succeed_on_third_try();
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(result.unwrap(), 3); // 3回目なので値は3
    }

    #[test]
    fn test_retry_always_succeeds() {
        let result = always_succeeds();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_always_fails() {
        let result = always_fails();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "always fails");
    }

    #[test]
    fn test_measure_time_sort() {
        let result = sort_numbers(vec![3, 1, 4, 1, 5, 9, 2, 6]);
        assert_eq!(result, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_measure_time_sum() {
        assert_eq!(sum_range(100), 5050);
    }

    #[test]
    fn test_complex_operation() {
        let data = vec![1, 2, 3, 4, 5];
        let result = complex_operation(data);
        // 1^2=1(<=10, 除外), 2^2=4(<=10, 除外), 3^2=9(<=10, 除外)
        // 4^2=16(>10, 含む), 5^2=25(>10, 含む)
        assert_eq!(result, vec![16, 25]);
    }

    #[test]
    fn test_conditionally_logged() {
        // debug_assertionsが有効の場合はlogged、そうでなければ普通の関数
        assert_eq!(conditionally_logged(5), 10);
    }

    #[test]
    fn test_important_operation() {
        let _ = important_operation(41); // must_useなので代入が必要
    }
}
