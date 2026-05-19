//! # 繰り返し（Repetition）
//!
//! `macro_rules!`の繰り返し演算子は可変長の引数リストを扱います。
//!
//! ## 繰り返し演算子
//!
//! | 演算子 | 意味 | 対応 |
//! |---|---|---|
//! | `$(...)*` | 0回以上 | `*` |
//! | `$(...)+` | 1回以上 | `+` |
//! | `$(...)?` | 0または1回 | `?` |
//!
//! ## セパレータ
//!
//! ```text
//! $($x:expr),*   // カンマ区切り
//! $($x:expr);+   // セミコロン区切り
//! $($x:expr)+    // 区切りなし
//! ```
//!
//! ## 展開側の繰り返し
//!
//! パターン側の繰り返し変数は展開側でも繰り返す必要があります:
//! ```text
//! ($($x:expr),*) => { [$($x * 2),*] }
//!                       ^^^^^^^  ← パターンと対応
//! ```

// ─── `$(...)*` ゼロ以上 ─────────────────────────────────────────────────────

/// カンマ区切りの値からVecを作る（標準ライブラリのvec!と同じ）
macro_rules! my_vec {
    ($($elem:expr),* $(,)?) => {
        // `$(,)?`で末尾カンマを許容
        vec![$($elem),*]
    };
}

/// 複数の値の合計を求めるマクロ
macro_rules! sum {
    ($($x:expr),*) => {{
        let mut total = 0;
        $(total += $x;)*
        total
    }};
}

/// HashMapリテラルを作るマクロ
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

// ─── `$(...)+` 1つ以上 ──────────────────────────────────────────────────────

/// 1つ以上の引数を受け取る最大値マクロ
macro_rules! max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        {
            let rest_max = max!($($rest),+);
            if $x > rest_max { $x } else { rest_max }
        }
    };
}

/// 最初の引数と残りの引数を分けて処理するマクロ
macro_rules! first_and_rest {
    ($first:expr $(, $rest:expr)+) => {
        ($first, vec![$($rest),+])
    };
}

// ─── `$(...)?` 0または1つ ────────────────────────────────────────────────────

/// オプション引数を持つマクロ
macro_rules! log_message {
    ($msg:expr) => {
        format!("[INFO] {}", $msg)
    };
    ($level:literal: $msg:expr) => {
        format!("[{}] {}", $level, $msg)
    };
}

/// デフォルト値付き引数のシミュレーション
macro_rules! connect {
    ($host:expr) => {
        connect!($host, 8080)
    };
    ($host:expr, $port:expr) => {
        format!("{}:{}", $host, $port)
    };
}

// ─── セパレータのバリエーション ──────────────────────────────────────────────

/// セミコロン区切りで複数の文を実行
macro_rules! exec_all {
    ($($stmt:expr);+ $(;)?) => {{
        $(let _ = $stmt;)+
    }};
}

/// 末尾カンマあり/なしどちらも対応
macro_rules! tuple_of {
    ($($x:expr),+ $(,)?) => {
        ($($x),+)
    };
}

// ─── ネストされた繰り返し ─────────────────────────────────────────────────────

/// 2次元配列（matrix）リテラルを作るマクロ
macro_rules! matrix {
    ($([$($elem:expr),+ $(,)?]),+ $(,)?) => {
        vec![$( vec![$($elem),+] ),+]
    };
}

/// 複数のトレイトを一度に実装するマクロ
macro_rules! impl_display_for {
    ($($ty:ty => $fmt:expr),+ $(,)?) => {
        $(
            impl std::fmt::Display for $ty {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, $fmt, self)
                }
            }
        )+
    };
}

// ─── 繰り返しのカウント ──────────────────────────────────────────────────────

/// 渡された引数の個数を返すマクロ
///
/// `replace_expr!`でトークンを`()`に置き換えてから配列長で数える手法。
macro_rules! replace_with_unit {
    ($e:expr) => { () };
}

macro_rules! count_args {
    ($($x:expr),*) => {{
        let arr: &[()] = &[$(replace_with_unit!($x)),*];
        arr.len()
    }};
}

// ─── 条件付き繰り返しパターン ───────────────────────────────────────────────

/// フィールドが存在する場合のみ初期化するマクロ
macro_rules! make_config {
    (
        host: $host:expr
        $(, port: $port:expr)?
        $(, timeout: $timeout:expr)?
        $(,)?
    ) => {{
        let host = $host.to_string();
        let port = 8080_u16 $(; let port = $port as u16; port)?;
        let timeout = 30_u64 $(; let timeout = $timeout as u64; timeout)?;
        (host, port, timeout)
    }};
}

// ─── 繰り返しによるenum生成 ──────────────────────────────────────────────────

/// 指定した変種を持つenum+Displayを生成するマクロ
macro_rules! simple_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $($variant),+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($name::$variant => write!(f, stringify!($variant))),+
                }
            }
        }
    };
}

simple_enum!(Color { Red, Green, Blue });
simple_enum!(Status { Active, Inactive, Pending });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_vec() {
        let v = my_vec![1, 2, 3];
        assert_eq!(v, vec![1, 2, 3]);

        let empty: Vec<i32> = my_vec![];
        assert!(empty.is_empty());

        // 末尾カンマも許容
        let v2 = my_vec![1, 2, 3,];
        assert_eq!(v2, vec![1, 2, 3]);
    }

    #[test]
    fn test_sum() {
        assert_eq!(sum!(1, 2, 3, 4, 5), 15);
        assert_eq!(sum!(10), 10);
        assert_eq!(sum!(), 0);
    }

    #[test]
    fn test_hashmap() {
        let m = hashmap! {
            "one" => 1,
            "two" => 2,
            "three" => 3,
        };
        assert_eq!(m["one"], 1);
        assert_eq!(m["three"], 3);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_max() {
        assert_eq!(max!(3), 3);
        assert_eq!(max!(1, 5, 3), 5);
        assert_eq!(max!(10, 2, 8, 1, 9), 10);
        assert_eq!(max!(-1, -5, -2), -1);
    }

    #[test]
    fn test_first_and_rest() {
        let (first, rest) = first_and_rest!(1, 2, 3, 4);
        assert_eq!(first, 1);
        assert_eq!(rest, vec![2, 3, 4]);
    }

    #[test]
    fn test_log_message() {
        assert_eq!(log_message!("hello"), "[INFO] hello");
        assert_eq!(log_message!("ERROR": "oh no"), "[ERROR] oh no");
    }

    #[test]
    fn test_connect() {
        assert_eq!(connect!("localhost"), "localhost:8080");
        assert_eq!(connect!("example.com", 443), "example.com:443");
    }

    #[test]
    fn test_matrix() {
        let m = matrix![
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9],
        ];
        assert_eq!(m.len(), 3);
        assert_eq!(m[0], vec![1, 2, 3]);
        assert_eq!(m[1][1], 5);
    }

    #[test]
    fn test_count_args() {
        assert_eq!(count_args!(), 0);
        assert_eq!(count_args!(1), 1);
        assert_eq!(count_args!(1, 2, 3), 3);
        assert_eq!(count_args!("a", "b", "c", "d"), 4);
    }

    #[test]
    fn test_tuple_of() {
        assert_eq!(tuple_of!(1, 2, 3), (1, 2, 3));
        assert_eq!(tuple_of!("hello", "world"), ("hello", "world"));
    }

    #[test]
    fn test_simple_enum() {
        assert_eq!(format!("{}", Color::Red), "Red");
        assert_eq!(format!("{}", Status::Active), "Active");
        assert_eq!(Color::Green, Color::Green);
        assert_ne!(Color::Blue, Color::Red);
    }

    #[test]
    fn test_make_config() {
        let (host, port, _timeout) = make_config!(host: "localhost");
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);

        let (host2, port2, timeout2) = make_config!(host: "server.com", port: 443, timeout: 60);
        assert_eq!(host2, "server.com");
        assert_eq!(port2, 443);
        assert_eq!(timeout2, 60);
    }

    #[test]
    fn test_nested_repetition() {
        let m = matrix![[1, 2], [3, 4]];
        assert_eq!(m[0][0], 1);
        assert_eq!(m[1][1], 4);
    }
}
