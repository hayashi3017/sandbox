//! # フラグメント指定子（Fragment Specifiers）
//!
//! `macro_rules!`のパターン内で使う型タグです。
//! 各フラグメント指定子は受け入れられる構文の種類を定義します。
//!
//! ## フラグメント指定子一覧
//!
//! | 指定子 | 受け入れる構文 | 例 |
//! |---|---|---|
//! | `expr` | 式 | `1 + 2`, `foo()`, `"hello"` |
//! | `ident` | 識別子 | `foo`, `my_var`, `SomeType` |
//! | `ty` | 型 | `i32`, `Vec<String>`, `&'a T` |
//! | `literal` | リテラル | `42`, `"hi"`, `3.14`, `true` |
//! | `block` | ブロック式 | `{ let x = 1; x }` |
//! | `stmt` | 文 | `let x = 1;`, `expr;` |
//! | `pat` / `pat_param` | パターン | `Some(x)`, `1..=10`, `_` |
//! | `item` | アイテム | `fn foo(){}`, `struct Bar{}` |
//! | `meta` | 属性のメタデータ | `derive(Debug)`, `cfg(test)` |
//! | `tt` | トークンツリー | 任意のトークン |
//! | `lifetime` | ライフタイム | `'a`, `'static` |
//! | `vis` | 可視性修飾子 | `pub`, `pub(crate)`, （空） |
//! | `path` | パス | `std::collections::HashMap` |
//!
//! ## フラグメントの後続制限（Follow Set）
//!
//! 特定のフラグメントの後に続けられるトークンに制限があります:
//! - `expr`, `stmt`: `=>`, `,`, `;`のみ
//! - `ty`, `path`: `=>`, `,`, `>`, `[`, `{`, `as`, `where`, ...のみ
//! - `pat`: `=>`, `,`, `=`, `|`, `if`, `in`のみ


// ─── expr ────────────────────────────────────────────────────────────────────

/// 任意の式を受け取るマクロ
///
/// 複雑な式（メソッド呼び出し、クロージャ等）も渡せる
macro_rules! print_result {
    ($e:expr) => {{
        let v = $e;
        format!("{} = {:?}", stringify!($e), v)
    }};
}

/// 式を受け取ってOption型で返すマクロ
macro_rules! try_eval {
    ($e:expr) => {
        std::panic::catch_unwind(|| $e).ok()
    };
}

// ─── ident ───────────────────────────────────────────────────────────────────

/// 識別子からgetterとsetterを生成するマクロ
macro_rules! getter_setter {
    ($field:ident : $ty:ty) => {
        paste::paste! {
            pub fn [<get_ $field>](&self) -> &$ty {
                &self.$field
            }
            pub fn [<set_ $field>](&mut self, val: $ty) {
                self.$field = val;
            }
        }
    };
}

/// 識別子を文字列化してフォーマット
macro_rules! ident_str {
    ($id:ident) => {
        stringify!($id)
    };
}

/// 複数の識別子を連結して定数名を作る（`paste`クレートなしの代替例）
macro_rules! make_const {
    ($prefix:ident, $suffix:ident, $val:expr) => {
        // ここでは単純に表示するデモとして使う
        // 実際の識別子連結はpaste!クレートが必要
        ($val, concat!(stringify!($prefix), "_", stringify!($suffix)))
    };
}

// ─── ty ──────────────────────────────────────────────────────────────────────

/// 型を受け取ってその情報を返すマクロ
macro_rules! type_info {
    ($t:ty) => {{
        (
            stringify!($t),
            std::mem::size_of::<$t>(),
            std::mem::align_of::<$t>(),
        )
    }};
}

/// 型を受け取ってデフォルト値を生成するマクロ
macro_rules! zero_of {
    ($t:ty) => {
        <$t as Default>::default()
    };
}

/// 複数の型のVecを作るマクロ
macro_rules! type_sizes {
    ($($t:ty),+) => {
        vec![$( (stringify!($t), std::mem::size_of::<$t>()) ),+]
    };
}

// ─── literal ─────────────────────────────────────────────────────────────────

/// リテラルのみを受け取るマクロ（変数や式は不可）
macro_rules! double_literal {
    ($x:literal) => {
        $x * 2
    };
}

/// 文字列リテラルを大文字化してコンパイル時定数にするデモ
/// （実際の変換はconcatなどで行う）
macro_rules! static_greeting {
    ($prefix:literal, $name:literal) => {
        concat!($prefix, ", ", $name, "!")
    };
}

// ─── block ───────────────────────────────────────────────────────────────────

/// ブロックをn回繰り返すマクロ（固定回数）
macro_rules! repeat_block {
    ($n:literal, $block:block) => {{
        for _ in 0..$n {
            $block
        }
    }};
}

/// ブロックの実行時間を計測するマクロ
macro_rules! time_it {
    ($label:literal, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        (result, elapsed, $label)
    }};
}

// ─── stmt ────────────────────────────────────────────────────────────────────

/// 複数の文を受け取ってブロックにまとめるマクロ
macro_rules! statements {
    ($($s:stmt);+ $(;)?) => {{
        $($s;)+
    }};
}

// ─── pat ─────────────────────────────────────────────────────────────────────

/// パターンを引数に取るマクロ（matchの簡略記法）
macro_rules! matches_pat {
    ($val:expr, $pat:pat) => {
        matches!($val, $pat)
    };
}

/// 複数パターンで値をチェックするマクロ
macro_rules! is_any_of {
    ($val:expr, $($pat:pat_param)|+) => {
        matches!($val, $($pat)|+)
    };
}

// ─── item ────────────────────────────────────────────────────────────────────

/// モジュール内にアイテムを配置するマクロ
macro_rules! private_module {
    ($($item:item)*) => {
        mod _private {
            #![allow(dead_code)]
            $($item)*
        }
    };
}

private_module! {
    fn hidden_fn() -> i32 { 42 }
    struct HiddenStruct { val: i32 }
}

// ─── meta ────────────────────────────────────────────────────────────────────

/// 属性メタデータを受け取るマクロ
macro_rules! with_attr {
    (#[$m:meta] $item:item) => {
        #[$m]
        $item
    };
}

with_attr! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Tagged { pub name: String, pub value: i32 }
}

// ─── tt（トークンツリー）────────────────────────────────────────────────────

/// 任意のトークンを受け取るマクロ（最も汎用）
///
/// `tt`はすべてのフラグメントを包括する最も汎用なマクロです。
/// 複雑なDSLの実装に使います。
macro_rules! identity {
    ($($t:tt)*) => {
        $($t)*
    };
}

/// JSONライクな構文のマクロ（ttを使ったDSL例）
macro_rules! json_val {
    (null) => { None::<i64> };
    (true)  => { Some(1_i64) };
    (false) => { Some(0_i64) };
    ($n:literal) => { Some($n as i64) };
}

// ─── lifetime ────────────────────────────────────────────────────────────────

/// ライフタイムアノテーション付き構造体を生成するマクロ
macro_rules! ref_wrapper {
    ($name:ident<$lt:lifetime, $ty:ty>) => {
        pub struct $name<$lt> {
            inner: &$lt $ty,
        }

        impl<$lt> $name<$lt> {
            pub fn new(val: &$lt $ty) -> Self {
                $name { inner: val }
            }
            pub fn get(&self) -> &$ty {
                self.inner
            }
        }
    };
}

ref_wrapper!(StrRef<'a, str>);
ref_wrapper!(I32Ref<'a, i32>);

// ─── vis ─────────────────────────────────────────────────────────────────────

/// 可視性を引数に取る構造体生成マクロ
macro_rules! make_newtype {
    ($vis:vis $name:ident($inner:ty)) => {
        $vis struct $name($inner);

        impl $name {
            $vis fn new(val: $inner) -> Self {
                $name(val)
            }
            $vis fn into_inner(self) -> $inner {
                self.0
            }
        }
    };
}

make_newtype!(pub UserId(u64));
make_newtype!(pub(crate) SessionToken(String));

// ─── path ────────────────────────────────────────────────────────────────────

/// パスを受け取るマクロ
macro_rules! use_trait {
    ($path:path, $ty:ty) => {
        <$ty as $path>::default()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_result() {
        let s = print_result!(1 + 2);
        assert!(s.contains("1 + 2"));
        assert!(s.contains('3'));
    }

    #[test]
    fn test_ident_str() {
        assert_eq!(ident_str!(my_variable), "my_variable");
        assert_eq!(ident_str!(HashMap), "HashMap");
    }

    #[test]
    fn test_type_info() {
        let (name, size, align) = type_info!(i32);
        assert_eq!(name, "i32");
        assert_eq!(size, 4);
        assert_eq!(align, 4);

        let (name64, size64, _) = type_info!(u64);
        assert_eq!(name64, "u64");
        assert_eq!(size64, 8);
    }

    #[test]
    fn test_zero_of() {
        assert_eq!(zero_of!(i32), 0);
        assert_eq!(zero_of!(f64), 0.0);
        assert_eq!(zero_of!(String), "");
        assert_eq!(zero_of!(Vec<i32>), vec![]);
    }

    #[test]
    fn test_type_sizes() {
        let sizes = type_sizes!(u8, u16, u32, u64);
        assert_eq!(sizes[0].1, 1);
        assert_eq!(sizes[1].1, 2);
        assert_eq!(sizes[2].1, 4);
        assert_eq!(sizes[3].1, 8);
    }

    #[test]
    fn test_double_literal() {
        assert_eq!(double_literal!(5), 10);
        assert_eq!(double_literal!(3), 6);
    }

    #[test]
    fn test_static_greeting() {
        const MSG: &str = static_greeting!("Hello", "Rust");
        assert_eq!(MSG, "Hello, Rust!");
    }

    #[test]
    fn test_matches_pat() {
        assert!(matches_pat!(Some(42), Some(_)));
        assert!(!matches_pat!(None::<i32>, Some(_)));
        assert!(matches_pat!(42, 40..=50));
    }

    #[test]
    fn test_is_any_of() {
        let x = 3_i32;
        assert!(is_any_of!(x, 1 | 2 | 3));
        assert!(!is_any_of!(x, 4 | 5 | 6));
    }

    #[test]
    fn test_tagged_struct() {
        let t = Tagged {
            name: "test".to_string(),
            value: 42,
        };
        let t2 = t.clone();
        assert_eq!(t, t2);
        assert_eq!(format!("{:?}", t), r#"Tagged { name: "test", value: 42 }"#);
    }

    #[test]
    fn test_json_val() {
        assert_eq!(json_val!(null), None);
        assert_eq!(json_val!(42), Some(42_i64));
        assert_eq!(json_val!(true), Some(1_i64));
    }

    #[test]
    fn test_identity() {
        let x = identity!(1 + 2 * 3);
        assert_eq!(x, 7);
    }

    #[test]
    fn test_ref_wrapper() {
        let s = "hello";
        let r = StrRef::new(s);
        assert_eq!(r.get(), "hello");

        let n = 42_i32;
        let ri = I32Ref::new(&n);
        assert_eq!(*ri.get(), 42);
    }

    #[test]
    fn test_make_newtype() {
        let id = UserId::new(12345);
        assert_eq!(id.into_inner(), 12345);
    }

    #[test]
    fn test_make_const() {
        let (val, name) = make_const!(MAX, SIZE, 100);
        assert_eq!(val, 100);
        assert_eq!(name, "MAX_SIZE");
    }

    #[test]
    fn test_use_trait() {
        let v: Vec<i32> = use_trait!(Default, Vec<i32>);
        assert!(v.is_empty());
    }
}
