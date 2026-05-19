//! # 関数形式手続きマクロ（Function-like Procedural Macros）
//!
//! 関数形式マクロは`macro_rules!`と同じ呼び出し構文（`macro!()`）を持ちますが、
//! 手続きマクロとして実装されるため、より複雑な構文解析が可能です。
//!
//! ## macro_rules! との違い
//!
//! | 特性 | `macro_rules!` | 関数形式proc_macro |
//! |---|---|---|
//! | 実装方法 | パターンマッチング | Rustコード |
//! | 構文解析 | パターン | `syn`等で任意 |
//! | エラーメッセージ | 基本的 | カスタム可能（`span!`等） |
//! | 複雑なDSL | 限界あり | 任意 |
//! | 別クレート必要 | 不要 | 必要（`proc-macro = true`） |
//!
//! ## このモジュールのカスタム関数形式マクロ
//!
//! | マクロ | 効果 |
//! |---|---|
//! | `count_tokens!(...)` | トークン数を返す |
//! | `sql!(SELECT ... FROM ...)` | SQLクエリを構造体に変換 |
//! | `make_enum!(Name { V1, V2 })` | enumと文字列変換を生成 |

use macros_impl::{count_tokens, make_enum, sql};

// ─── count_tokens! ────────────────────────────────────────────────────────────

pub fn count_demo() -> Vec<usize> {
    vec![
        count_tokens!(),
        count_tokens!(a),
        count_tokens!(a b c),
        count_tokens!(1 + 2),      // 3トークン: 1, +, 2
        count_tokens!(hello, world), // 3トークン: hello, ',', world
    ]
}

// ─── sql! ────────────────────────────────────────────────────────────────────

pub fn sql_demo() {
    let q = sql!(SELECT name, age FROM users);
    assert_eq!(q.table, "users");
    assert_eq!(q.columns, vec!["name", "age"]);

    let q2 = sql!(SELECT id FROM products);
    assert_eq!(q2.table, "products");
    assert_eq!(q2.columns, vec!["id"]);
}

// ─── make_enum! ──────────────────────────────────────────────────────────────

/// 関数形式マクロで生成されたenum
make_enum!(Direction { North, South, East, West });
make_enum!(Status { Active, Inactive, Pending, Deleted });
make_enum!(LogLevel { Debug, Info, Warn, Error });

// ─── 標準ライブラリの関数形式マクロとの比較 ──────────────────────────────────

/// 標準ライブラリの主な関数形式マクロ
pub fn stdlib_function_like_macros() -> Vec<(&'static str, String)> {
    vec![
        ("format!", format!("{} + {} = {}", 1, 2, 3)),
        ("concat!", concat!("Hello", ", ", "World!").to_string()),
        ("stringify!", stringify!(x + y * z).to_string()),
        ("env!", option_env!("HOME").unwrap_or("unknown").to_string()),
        ("cfg!", cfg!(debug_assertions).to_string()),
        ("file!", file!().to_string()),
        ("line!", line!().to_string()),
        ("column!", column!().to_string()),
        ("module_path!", module_path!().to_string()),
    ]
}

// ─── DSL実装の実例：設定ファイルパーサー ────────────────────────────────────

/// `make_enum!`で生成したenumの活用例
pub fn use_generated_enums() -> Vec<String> {
    let directions = vec![
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    let log_levels = vec![
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ];

    let mut results = Vec::new();

    for d in &directions {
        results.push(format!("Direction: {}", d));
    }

    for l in &log_levels {
        results.push(format!("LogLevel: {} ({:?})", l, l));
    }

    results
}

/// 生成されたenumで状態管理
pub fn direction_opposite(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East  => Direction::West,
        Direction::West  => Direction::East,
    }
}

pub fn is_error_or_warn(level: LogLevel) -> bool {
    matches!(level, LogLevel::Error | LogLevel::Warn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        let counts = count_demo();
        assert_eq!(counts[0], 0); // 空
        assert_eq!(counts[1], 1); // a
        assert_eq!(counts[2], 3); // a b c
        assert_eq!(counts[3], 3); // 1 + 2
    }

    #[test]
    fn test_sql_basic() {
        let q = sql!(SELECT name, age FROM users);
        assert_eq!(q.table, "users");
        assert_eq!(q.columns, vec!["name", "age"]);
    }

    #[test]
    fn test_sql_single_column() {
        let q = sql!(SELECT id FROM products);
        assert_eq!(q.table, "products");
        assert_eq!(q.columns, vec!["id"]);
    }

    #[test]
    fn test_sql_demo_no_panic() {
        sql_demo(); // パニックしなければOK
    }

    #[test]
    fn test_make_enum_as_str() {
        assert_eq!(Direction::North.as_str(), "North");
        assert_eq!(Direction::South.as_str(), "South");
        assert_eq!(Direction::East.as_str(), "East");
        assert_eq!(Direction::West.as_str(), "West");
    }

    #[test]
    fn test_make_enum_display() {
        assert_eq!(format!("{}", Direction::North), "North");
        assert_eq!(format!("{}", Status::Active), "Active");
        assert_eq!(format!("{}", LogLevel::Error), "Error");
    }

    #[test]
    fn test_make_enum_debug() {
        assert_eq!(format!("{:?}", Direction::North), "North");
        assert_eq!(format!("{:?}", LogLevel::Warn), "Warn");
    }

    #[test]
    fn test_make_enum_equality() {
        assert_eq!(Direction::North, Direction::North);
        assert_ne!(Direction::North, Direction::South);
    }

    #[test]
    fn test_make_enum_copy() {
        // make_enum!はCopyを実装するのでムーブなしで使える
        let d = Direction::East;
        let d2 = d; // Copy
        assert_eq!(d, d2);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(direction_opposite(Direction::North), Direction::South);
        assert_eq!(direction_opposite(Direction::East), Direction::West);
        assert_eq!(direction_opposite(Direction::South), Direction::North);
    }

    #[test]
    fn test_is_error_or_warn() {
        assert!(is_error_or_warn(LogLevel::Error));
        assert!(is_error_or_warn(LogLevel::Warn));
        assert!(!is_error_or_warn(LogLevel::Info));
        assert!(!is_error_or_warn(LogLevel::Debug));
    }

    #[test]
    fn test_status_variants() {
        let statuses = vec![
            Status::Active,
            Status::Inactive,
            Status::Pending,
            Status::Deleted,
        ];
        let names: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
        assert_eq!(names, vec!["Active", "Inactive", "Pending", "Deleted"]);
    }

    #[test]
    fn test_use_generated_enums() {
        let results = use_generated_enums();
        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.contains("North")));
        assert!(results.iter().any(|s| s.contains("Error")));
    }

    #[test]
    fn test_stdlib_macros() {
        let macros = stdlib_function_like_macros();
        assert!(!macros.is_empty());
        let fmt_result = &macros[0];
        assert_eq!(fmt_result.0, "format!");
        assert_eq!(fmt_result.1, "1 + 2 = 3");
    }
}
