//! # `macro_rules!` の基礎
//!
//! 宣言的マクロはパターンマッチングで動作します。
//! 入力トークン列がパターンに一致すると、対応する出力に展開されます。
//!
//! ## 基本構文
//!
//! ```text
//! macro_rules! マクロ名 {
//!     (パターン1) => { 展開1 };
//!     (パターン2) => { 展開2 };
//! }
//! ```
//!
//! ## 呼び出し構文（3種類）
//!
//! ```text
//! macro_name!()   // 丸括弧
//! macro_name![]   // 角括弧
//! macro_name! {}  // 波括弧
//! ```
//!
//! 3つの呼び出し形式はすべて等価です（慣習的に`vec![]`、`println!()`のように使い分けます）。

// ─── 最もシンプルなマクロ ───────────────────────────────────────────────────

/// 引数なしのマクロ
macro_rules! say_hello {
    () => {
        "Hello, macro!"
    };
}

// ─── 単一引数 ───────────────────────────────────────────────────────────────

/// 式を受け取って2倍にするマクロ
macro_rules! double {
    ($x:expr) => {
        $x * 2
    };
}

/// 式を受け取って文字列化するマクロ
macro_rules! expr_to_string {
    ($e:expr) => {
        stringify!($e)
    };
}

// ─── 複数のパターン（アーム） ────────────────────────────────────────────────

/// 引数の数で動作が変わるマクロ（オーバーロード）
macro_rules! greet {
    () => {
        "Hello!".to_string()
    };
    ($name:expr) => {
        format!("Hello, {}!", $name)
    };
    ($greeting:expr, $name:expr) => {
        format!("{}, {}!", $greeting, $name)
    };
}

// ─── 識別子の生成 ────────────────────────────────────────────────────────────

/// 関数を生成するマクロ
///
/// `$func_name:ident`で識別子（変数名・関数名）を受け取り、
/// `stringify!`でその名前を文字列に変換する。
macro_rules! create_function {
    ($func_name:ident) => {
        pub fn $func_name() -> &'static str {
            stringify!($func_name)
        }
    };
}

create_function!(generated_foo);
create_function!(generated_bar);

// ─── 型の受け渡し ────────────────────────────────────────────────────────────

/// 型引数を受け取るマクロ
macro_rules! create_vec_of {
    ($t:ty) => {
        Vec::<$t>::new()
    };
}

/// デフォルト値付きの型を初期化するマクロ
macro_rules! default_value {
    (i32) => { 0_i32 };
    (f64) => { 0.0_f64 };
    (bool) => { false };
    (String) => { String::new() };
    ($t:ty) => { <$t>::default() };
}

// ─── 構造体・implの生成 ──────────────────────────────────────────────────────

/// Named structとコンストラクタを生成するマクロ
macro_rules! define_struct {
    ($name:ident { $($field:ident : $ty:ty),+ $(,)? }) => {
        #[derive(Debug, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),+
        }

        impl $name {
            pub fn new($($field: $ty),+) -> Self {
                $name { $($field),+ }
            }
        }
    };
}

define_struct!(Point2D { x: f64, y: f64 });
define_struct!(Point3D { x: f64, y: f64, z: f64 });
define_struct!(Color { r: u8, g: u8, b: u8 });

// ─── カスタムアサートマクロ ──────────────────────────────────────────────────

/// 詳細なエラーメッセージを出すassertマクロ
macro_rules! assert_close {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let l = $left;
        let r = $right;
        let eps = $epsilon;
        if (l - r).abs() > eps {
            panic!(
                "assertion failed: |{} - {}| <= {}\n  left:  {:?}\n  right: {:?}",
                stringify!($left),
                stringify!($right),
                eps,
                l,
                r
            );
        }
    }};
}

// ─── セミコロン区切りの複数文 ─────────────────────────────────────────────────

/// 複数の式を順に評価して最後の値を返すマクロ
macro_rules! eval_seq {
    ($e:expr) => { $e };
    ($first:expr; $($rest:expr);+) => {{
        let _ = $first;
        eval_seq!($($rest);+)
    }};
}

// ─── マクロで定数を生成 ──────────────────────────────────────────────────────

/// ビットマスク定数を生成するマクロ
macro_rules! bitmask {
    ($name:ident, $bit:expr) => {
        pub const $name: u32 = 1 << $bit;
    };
}

bitmask!(FLAG_A, 0);
bitmask!(FLAG_B, 1);
bitmask!(FLAG_C, 2);
bitmask!(FLAG_ALL, 7);

// ─── マッチ式の拡張 ─────────────────────────────────────────────────────────

/// 列挙型の変種名を文字列で返すマクロ
macro_rules! enum_name {
    ($val:expr, { $($variant:pat => $name:literal),+ $(,)? }) => {
        match $val {
            $($variant => $name),+
        }
    };
}

#[derive(Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub fn direction_name(d: &Direction) -> &'static str {
    enum_name!(d, {
        Direction::North => "North",
        Direction::South => "South",
        Direction::East  => "East",
        Direction::West  => "West",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_say_hello() {
        assert_eq!(say_hello!(), "Hello, macro!");
    }

    #[test]
    fn test_double() {
        assert_eq!(double!(5), 10);
        assert_eq!(double!(0), 0);
        assert_eq!(double!(-3), -6);
        assert_eq!(double!(2 + 3), 10); // 式も渡せる
    }

    #[test]
    fn test_expr_to_string() {
        assert_eq!(expr_to_string!(1 + 2), "1 + 2");
        assert_eq!(expr_to_string!(x * y), "x * y");
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet!(), "Hello!");
        assert_eq!(greet!("World"), "Hello, World!");
        assert_eq!(greet!("Hi", "Rust"), "Hi, Rust!");
    }

    #[test]
    fn test_create_function() {
        assert_eq!(generated_foo(), "generated_foo");
        assert_eq!(generated_bar(), "generated_bar");
    }

    #[test]
    fn test_create_vec_of() {
        let v: Vec<i32> = create_vec_of!(i32);
        assert!(v.is_empty());

        let v2: Vec<String> = create_vec_of!(String);
        assert!(v2.is_empty());
    }

    #[test]
    fn test_default_value() {
        assert_eq!(default_value!(i32), 0);
        assert_eq!(default_value!(bool), false);
        assert_eq!(default_value!(String), "");
    }

    #[test]
    fn test_define_struct_point2d() {
        let p = Point2D::new(3.0, 4.0);
        assert_eq!(p.x, 3.0);
        assert_eq!(p.y, 4.0);
    }

    #[test]
    fn test_define_struct_point3d() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.z, 3.0);
    }

    #[test]
    fn test_define_struct_color() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_assert_close() {
        assert_close!(1.0_f64, 1.0000001, 0.001);
        assert_close!(3.14, std::f64::consts::PI, 0.01);
    }

    #[test]
    #[should_panic]
    fn test_assert_close_fail() {
        assert_close!(1.0_f64, 2.0, 0.001);
    }

    #[test]
    fn test_eval_seq() {
        assert_eq!(eval_seq!(1 + 2), 3);
        assert_eq!(eval_seq!(1; 2; 3), 3);
    }

    #[test]
    fn test_bitmasks() {
        assert_eq!(FLAG_A, 0b0001);
        assert_eq!(FLAG_B, 0b0010);
        assert_eq!(FLAG_C, 0b0100);
        assert!(FLAG_A & FLAG_B == 0);
    }

    #[test]
    fn test_delimiter_forms() {
        // 3種類の呼び出し形式はすべて等価
        assert_eq!(double!(4), double![4]);
        assert_eq!(double!(4), double! { 4 });
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(direction_name(&Direction::North), "North");
        assert_eq!(direction_name(&Direction::East), "East");
    }
}
