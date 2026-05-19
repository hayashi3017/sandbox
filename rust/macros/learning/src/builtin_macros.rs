//! # 標準ライブラリの組み込みマクロ
//!
//! Rustには多数の組み込みマクロが含まれています。
//! それぞれの用途と動作を確認します。
//!
//! ## カテゴリ
//!
//! | カテゴリ | マクロ |
//! |---|---|
//! | 出力 | `println!`, `print!`, `eprintln!`, `eprint!` |
//! | フォーマット | `format!`, `write!`, `writeln!`, `format_args!` |
//! | デバッグ | `dbg!`, `todo!`, `unimplemented!`, `unreachable!` |
//! | アサート | `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` |
//! | パニック | `panic!` |
//! | コレクション | `vec!` |
//! | 文字列 | `concat!`, `stringify!`, `include_str!`, `include_bytes!` |
//! | 環境 | `env!`, `option_env!`, `file!`, `line!`, `column!`, `module_path!` |
//! | コンパイル制御 | `cfg!`, `compile_error!` |
//! | その他 | `matches!`, `is_x86_feature_detected!` |

use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

// ─── フォーマット系 ──────────────────────────────────────────────────────────

pub fn format_examples() -> Vec<String> {
    vec![
        // 基本フォーマット
        format!("Hello, {}!", "world"),
        // 幅・精度指定
        format!("{:>10}", "right"),    // 右寄せ幅10
        format!("{:<10}", "left"),     // 左寄せ幅10
        format!("{:^10}", "center"),   // 中央寄せ幅10
        format!("{:0>5}", 42),         // ゼロパディング
        format!("{:.3}", 3.14159),     // 小数点以下3桁
        // 進数フォーマット
        format!("{:b}", 42),           // 2進数
        format!("{:o}", 42),           // 8進数
        format!("{:x}", 255),          // 16進数(小文字)
        format!("{:X}", 255),          // 16進数(大文字)
        format!("{:#b}", 42),          // 0b付き2進数
        format!("{:#x}", 255),         // 0x付き16進数
        // デバッグフォーマット
        format!("{:?}", vec![1, 2, 3]),
        format!("{:#?}", (1, "hello")), // pretty-print
        // 名前付き引数
        format!("{name} is {age}", name = "Alice", age = 30),
        // インデックス指定
        format!("{0} {1} {0}", "a", "b"),
    ]
}

// ─── write! / writeln! ───────────────────────────────────────────────────────

pub fn write_to_string() -> String {
    let mut s = String::new();
    // Stringへのwrite!にはstd::fmt::Writeトレイトが必要
    FmtWrite::write_fmt(&mut s, format_args!("Hello")).unwrap();
    write!(s, ", {}!", "world").unwrap(); // use std::fmt::Write で使えるようになる
    writeln!(s, " Done.").unwrap();
    s
}

pub fn write_to_vec() -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    // Vec<u8>へのwrite!にはstd::io::Writeトレイトが必要
    IoWrite::write_fmt(&mut buf, format_args!("binary data: ")).unwrap();
    writeln!(&mut buf, "{}", 42).unwrap();
    buf
}

// ─── dbg! ────────────────────────────────────────────────────────────────────

/// `dbg!`は値を標準エラーに表示しつつ、値を返す
///
/// デバッグ時に便利。`println!`と異なり式の結果を保持。
pub fn dbg_demo() -> i32 {
    // dbg!は式の評価結果を返すので計算の途中に挿入できる
    let x = 1;
    let y = 2;
    // dbg!(x + y) は stderr に "[src/builtin_macros.rs:70] x + y = 3" を表示し、3を返す
    let _sum = dbg!(x + y); // テスト時はstderrに出力される
    x + y
}

// ─── アサートマクロ ──────────────────────────────────────────────────────────

pub fn assert_examples() {
    // 基本アサート
    assert!(1 + 1 == 2);
    assert!(true, "This should never fail");

    // 等値アサート（失敗時に両方の値を表示）
    assert_eq!(2 + 2, 4);
    assert_eq!("hello".len(), 5, "String length mismatch");

    // 非等値アサート
    assert_ne!(1, 2);
    assert_ne!(Vec::<i32>::new(), vec![1, 2, 3]);

    // デバッグビルドのみ実行されるアサート（リリースビルドでは無視）
    debug_assert!(1 < 2);
    debug_assert_eq!(3 * 3, 9);
    debug_assert_ne!(0, 1);
}

// ─── todo! / unimplemented! / unreachable! ──────────────────────────────────

/// `todo!()`はプレースホルダー（実装中の印）
/// コンパイルは通るがランタイムでpanicする
pub fn not_yet_done() -> i32 {
    if false {
        todo!("この分岐はまだ実装していない")
    } else {
        42
    }
}

/// `unimplemented!()`はサポートしない機能のマーカー
pub fn unsupported_operation(op: &str) -> i32 {
    match op {
        "double" => 2,
        _ => {
            if false {
                unimplemented!("Operation '{}' is not supported", op)
            } else {
                0
            }
        }
    }
}

/// `unreachable!()`は到達不可能なコードのマーカー
pub fn direction_to_degrees(dir: &str) -> u32 {
    match dir {
        "N" => 0,
        "E" => 90,
        "S" => 180,
        "W" => 270,
        _ => unreachable!("Invalid direction: {}", dir),
    }
}

// ─── 文字列系マクロ ─────────────────────────────────────────────────────────

/// `concat!`はコンパイル時に文字列を結合する
const GREETING: &str = concat!("Hello", ", ", "Rust", "!");

/// `stringify!`はトークンを文字列化する
pub fn stringify_examples() -> Vec<&'static str> {
    vec![
        stringify!(1 + 2),
        stringify!(let x = 5),
        stringify!(HashMap<String, Vec<i32>>),
    ]
}

/// `include_str!`はファイルをコンパイル時に文字列として埋め込む
/// （実際のファイルが存在する場合に使用可能）
/// ```text
/// const README: &str = include_str!("../README.md");
/// ```

/// `include_bytes!`はファイルをバイト配列として埋め込む
/// ```text
/// const ICON: &[u8] = include_bytes!("icon.png");
/// ```

// ─── 環境・位置情報マクロ ────────────────────────────────────────────────────

pub fn location_info() -> (&'static str, u32, u32, &'static str) {
    (
        file!(),    // ファイルパス
        line!(),    // 行番号
        column!(),  // 列番号
        module_path!(), // モジュールパス
    )
}

/// `env!`はコンパイル時に環境変数を取得する
/// ```text
/// const HOME: &str = env!("HOME"); // コンパイル時のHOME
/// ```
///
/// `option_env!`は環境変数が存在しない場合Noneを返す（コンパイルエラーにならない）
pub fn get_optional_env() -> Option<&'static str> {
    option_env!("OPTIONAL_VAR_THAT_MIGHT_NOT_EXIST")
}

// ─── cfg! ──────────────────────────────────────────────────────────────────

pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

pub fn platform_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    }
}

pub fn pointer_width() -> u32 {
    if cfg!(target_pointer_width = "64") {
        64
    } else if cfg!(target_pointer_width = "32") {
        32
    } else {
        0
    }
}

// ─── matches! ───────────────────────────────────────────────────────────────

pub fn matches_demo() -> Vec<bool> {
    let v: Vec<Option<i32>> = vec![Some(1), None, Some(3), None, Some(5)];

    vec![
        matches!(Some(42), Some(_)),       // true
        matches!(None::<i32>, Some(_)),    // false
        matches!(42_i32, 1..=50),          // true（範囲パターン）
        matches!(42_i32, 1 | 42 | 100),   // true（orパターン）
        v.iter().filter(|x| matches!(x, Some(_))).count() == 3, // 3個のSome
    ]
}

// ─── vec! の内部動作 ────────────────────────────────────────────────────────

/// `vec!`マクロの実装に近い手動版
macro_rules! my_vec2 {
    () => {
        Vec::new()
    };
    ($elem:expr; $n:expr) => {{
        let mut v = Vec::with_capacity($n);
        v.resize($n, $elem);
        v
    }};
    ($($x:expr),+ $(,)?) => {{
        let mut v = Vec::with_capacity(count_items!($($x),+));
        $(v.push($x);)+
        v
    }};
}

macro_rules! count_items {
    ($($x:expr),*) => { <[()]>::len(&[$(replace_expr!($x, ())),*]) };
}

macro_rules! replace_expr {
    ($_t:expr, $sub:expr) => { $sub };
}

pub fn vec_macro_demo() -> Vec<Vec<i32>> {
    vec![
        vec![1, 2, 3],               // 個別要素
        vec![0; 5],                  // 同値繰り返し
        my_vec2![10, 20, 30],        // 手動版
        my_vec2![99; 3],             // 手動版（繰り返し）
    ]
}

// ─── format_args! ───────────────────────────────────────────────────────────

/// `format_args!`はアロケーションなしにフォーマット引数を作る
///
/// ロギングなどでパフォーマンスが重要な場合に使う。
pub fn format_args_demo() -> String {
    let args = format_args!("value: {}", 42);
    args.to_string()
}

// ─── 実装マクロの参考: 標準ライブラリ版 vec! ────────────────────────────────
//
// 実際の`vec!`の実装（簡略版）:
//
// macro_rules! vec {
//     () => { $crate::vec::Vec::new() };
//     ($elem:expr; $n:expr) => {{
//         let mut v = $crate::vec::Vec::new();
//         v.resize_with($n, || $elem);
//         v
//     }};
//     ($($x:expr),+ $(,)?) => {{
//         $crate::slice::into_vec(
//             $crate::boxed::Box::new([$($x),+])
//         )
//     }};
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_examples() {
        let examples = format_examples();
        assert!(!examples.is_empty());
        assert_eq!(examples[0], "Hello, world!");
        assert_eq!(examples[4], "00042");      // {:0>5}
        assert_eq!(examples[5], "3.142");      // {:.3}
        assert_eq!(examples[6], "101010");     // {:b}  42 in binary
        assert_eq!(examples[8], "ff");         // {:x}
        assert_eq!(examples[9], "FF");         // {:X}
    }

    #[test]
    fn test_write_to_string() {
        let s = write_to_string();
        assert!(s.contains("Hello"));
        assert!(s.contains("world"));
    }

    #[test]
    fn test_write_to_vec() {
        let v = write_to_vec();
        let s = String::from_utf8(v).unwrap();
        assert!(s.contains("42"));
    }

    #[test]
    fn test_dbg_demo() {
        assert_eq!(dbg_demo(), 3);
    }

    #[test]
    fn test_assert_examples() {
        assert_examples(); // パニックしなければOK
    }

    #[test]
    fn test_not_yet_done() {
        assert_eq!(not_yet_done(), 42);
    }

    #[test]
    fn test_direction_to_degrees() {
        assert_eq!(direction_to_degrees("N"), 0);
        assert_eq!(direction_to_degrees("E"), 90);
        assert_eq!(direction_to_degrees("S"), 180);
        assert_eq!(direction_to_degrees("W"), 270);
    }

    #[test]
    #[should_panic]
    fn test_direction_invalid() {
        direction_to_degrees("X");
    }

    #[test]
    fn test_concat() {
        assert_eq!(GREETING, "Hello, Rust!");
    }

    #[test]
    fn test_stringify() {
        let strs = stringify_examples();
        assert_eq!(strs[0], "1 + 2");
        assert!(strs[2].contains("HashMap"));
    }

    #[test]
    fn test_location_info() {
        let (file, line, _col, module) = location_info();
        assert!(file.ends_with(".rs"));
        assert!(line > 0);
        assert!(module.contains("builtin_macros"));
    }

    #[test]
    fn test_cfg() {
        // cfg!はコンパイル時に評価されるので実行時の条件分岐は不要
        let _debug = is_debug_build();
        let platform = platform_name();
        assert!(["linux", "macos", "windows", "unknown"].contains(&platform));

        let width = pointer_width();
        assert!(width == 32 || width == 64);
    }

    #[test]
    fn test_matches_demo() {
        let results = matches_demo();
        assert!(results[0]); // Some(42) matches Some(_)
        assert!(!results[1]); // None doesn't match Some(_)
        assert!(results[2]); // 42 in 1..=50
        assert!(results[3]); // 42 matches 1 | 42 | 100
        assert!(results[4]); // 3 Somes in vec
    }

    #[test]
    fn test_vec_macro_demo() {
        let vecs = vec_macro_demo();
        assert_eq!(vecs[0], vec![1, 2, 3]);
        assert_eq!(vecs[1], vec![0, 0, 0, 0, 0]);
        assert_eq!(vecs[2], vec![10, 20, 30]);
        assert_eq!(vecs[3], vec![99, 99, 99]);
    }

    #[test]
    fn test_format_args_demo() {
        assert_eq!(format_args_demo(), "value: 42");
    }
}
