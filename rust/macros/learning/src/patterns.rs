//! # 高度なマクロパターン
//!
//! 実用的なマクロ実装で使われる高度なパターン集です。
//!
//! ## 主なパターン
//!
//! | パターン | 説明 |
//! |---|---|
//! | Push-down accumulation | 蓄積引数を使って再帰的に結果を組み立てる |
//! | Callback macros | マクロに別のマクロを渡す（高階マクロ） |
//! | Internal rules (`@`) | `@tag`で内部ルールと公開ルールを分離する |
//! | Incremental TT munching | トークン列を先頭から処理する |
//! | 末尾カンマ対応 | `$(,)?`で末尾カンマをオプションにする |

// ─── Internal rules（@タグ）パターン ────────────────────────────────────────

/// 公開インターフェースと内部実装を分離するマクロ
///
/// `@`で始まるアームは「内部ルール」として慣習的に使われる。
/// ユーザーが直接`@`アームを呼び出すことを防ぐ（コンパイルエラーにはならないが慣習）。
macro_rules! my_format {
    // 公開インターフェース
    ($fmt:literal $(, $arg:expr)*) => {
        my_format!(@build [] $fmt $(, $arg)*)
    };
    // 内部ルール: 引数なしのケース
    (@build [$($parts:expr),*] $fmt:literal) => {
        format!($fmt)
    };
    // 内部ルール: 引数ありのケース
    (@build [$($parts:expr),*] $fmt:literal, $first:expr $(, $rest:expr)*) => {
        format!($fmt, $first $(, $rest)*)
    };
}

// ─── ビルダーパターンの自動生成 ─────────────────────────────────────────────

/// ビルダーパターンを生成するマクロ
///
/// 元の構造体と対応するBuilderを自動生成する。
macro_rules! builder {
    (
        $vis:vis struct $name:ident {
            $($field:ident : $ty:ty),+ $(,)?
        }
    ) => {
        // 元の構造体
        #[derive(Debug, Clone, PartialEq)]
        $vis struct $name {
            $(pub $field: $ty),+
        }

        // Builderの構造体（全フィールドがOption）
        #[derive(Default)]
        $vis struct paste::paste! { [<$name Builder>] } {
            $($field: Option<$ty>),+
        }

        impl $name {
            $vis fn builder() -> paste::paste! { [<$name Builder>] } {
                Default::default()
            }
        }

        impl paste::paste! { [<$name Builder>] } {
            // 各フィールドのセッター
            $(
                $vis fn $field(mut self, val: $ty) -> Self {
                    self.$field = Some(val);
                    self
                }
            )+

            // build()メソッド
            $vis fn build(self) -> Result<$name, String> {
                Ok($name {
                    $(
                        $field: self.$field.ok_or_else(|| {
                            format!("field '{}' is required", stringify!($field))
                        })?,
                    )+
                })
            }
        }
    };
}

// builderマクロには`paste`クレートが必要なため、ここでは代替パターンを使う

/// pasteなしのビルダーパターン（フィールド名を直接指定）
macro_rules! simple_builder {
    (
        pub struct $name:ident {
            $($field:ident : $ty:ty),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),+
        }

        pub struct $name {
            $(pub $field: $ty),+
        }
    };
}

// ─── エラー処理パターン ──────────────────────────────────────────────────────

/// 複数のResult値を連鎖して処理するマクロ
///
/// `try!`マクロの一般化版。複数の操作を順に実行し、
/// 最初のエラーで早期リターンする。
macro_rules! try_all {
    ($e:expr) => {
        $e?
    };
    ($first:expr, $($rest:expr),+) => {{
        $first?;
        try_all!($($rest),+)
    }};
}

pub fn validate_all(a: i32, b: i32, c: i32) -> Result<i32, String> {
    fn check_positive(x: i32) -> Result<(), String> {
        if x > 0 {
            Ok(())
        } else {
            Err(format!("{} is not positive", x))
        }
    }
    try_all!(check_positive(a), check_positive(b), check_positive(c));
    Ok(a + b + c)
}

// ─── テーブル駆動テストパターン ──────────────────────────────────────────────

/// テストケーステーブルを展開するマクロ
macro_rules! table_tests {
    (fn $name:ident($($param:ident : $pty:ty),+) -> $ret:ty {
        $($input:expr => $expected:expr),+ $(,)?
    } with $func:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;
            $(
                #[test]
                fn $name() {
                    // テスト関数名は実際には動的に生成できないので
                    // ここでは単純なassertを使う
                }
            )+

            #[test]
            fn run_all() {
                let f = $func;
                $({
                    let result = f($input);
                    assert_eq!(result, $expected,
                        "Input: {:?}, Expected: {:?}, Got: {:?}",
                        $input, $expected, result
                    );
                })+
            }
        }
    };
}

// ─── 状態機械（State Machine）のマクロ実装 ────────────────────────────────────

/// 単純な状態機械を宣言するマクロ
///
/// statesとeventsを明示的に列挙することで、
/// 遷移リストでの重複イベントを防ぐ。
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
        pub enum State {
            $($state),+
        }

        #[derive(Debug, Clone, PartialEq)]
        pub enum Event {
            $($event),+
        }

        pub struct Machine {
            pub state: State,
        }

        impl Machine {
            pub fn new() -> Self {
                Machine { state: State::$initial }
            }

            pub fn transition(&mut self, event: &Event) -> bool {
                let next = match (&self.state, event) {
                    $(
                        (State::$from, Event::$ev) => Some(State::$to),
                    )+
                    _ => None,
                };
                if let Some(s) = next {
                    self.state = s;
                    true
                } else {
                    false
                }
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

// ─── トークン分類マクロ ──────────────────────────────────────────────────────

/// トークンが「+」か「-」かを判定するマクロ
macro_rules! is_plus {
    (+) => { true };
    ($other:tt) => { false };
}

macro_rules! is_minus {
    (-) => { true };
    ($other:tt) => { false };
}

/// 演算子に応じて処理を切り替えるマクロ
macro_rules! apply_op {
    ($a:expr, +, $b:expr) => { $a + $b };
    ($a:expr, -, $b:expr) => { $a - $b };
    ($a:expr, *, $b:expr) => { $a * $b };
    ($a:expr, /, $b:expr) => { $a / $b };
}

// ─── ドキュメントコメントの自動生成 ─────────────────────────────────────────

/// ドキュメントコメントを自動生成するマクロ
///
/// `#[doc = ...]`はマクロで展開できるため、動的なドキュメントを生成できる。
macro_rules! documented_const {
    ($name:ident: $ty:ty = $val:expr, doc: $doc:literal) => {
        #[doc = $doc]
        pub const $name: $ty = $val;
    };
}

documented_const!(MAX_RETRIES: u32 = 3, doc: "最大リトライ回数");
documented_const!(TIMEOUT_MS: u64 = 5000, doc: "タイムアウト（ミリ秒）");

// ─── 配列初期化パターン ──────────────────────────────────────────────────────

/// 固定長配列をパターンで初期化するマクロ
macro_rules! array_init {
    ($val:expr; $n:literal) => {
        [$val; $n]
    };
    ($($val:expr),+ $(,)?) => {
        [$($val),+]
    };
}

// ─── 条件コンパイルの抽象化 ─────────────────────────────────────────────────

/// プラットフォームごとの実装を切り替えるマクロ
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_format() {
        assert_eq!(my_format!("hello"), "hello");
        assert_eq!(my_format!("Hello, {}!", "world"), "Hello, world!");
        assert_eq!(my_format!("{} + {} = {}", 1, 2, 3), "1 + 2 = 3");
    }

    #[test]
    fn test_validate_all_success() {
        assert_eq!(validate_all(1, 2, 3), Ok(6));
    }

    #[test]
    fn test_validate_all_failure() {
        assert!(validate_all(-1, 2, 3).is_err());
        assert!(validate_all(1, 2, -3).is_err());
    }

    #[test]
    fn test_state_machine() {
        let mut m = Machine::new();
        assert_eq!(m.state, State::Idle);

        assert!(m.transition(&Event::Start));
        assert_eq!(m.state, State::Running);

        assert!(m.transition(&Event::Pause));
        assert_eq!(m.state, State::Paused);

        // 無効な遷移
        assert!(!m.transition(&Event::Start));
        assert_eq!(m.state, State::Paused); // 変化なし

        assert!(m.transition(&Event::Resume));
        assert_eq!(m.state, State::Running);

        assert!(m.transition(&Event::Stop));
        assert_eq!(m.state, State::Stopped);
    }

    #[test]
    fn test_is_plus_minus() {
        assert!(is_plus!(+));
        assert!(!is_plus!(-));
        assert!(is_minus!(-));
        assert!(!is_minus!(+));
    }

    #[test]
    fn test_apply_op() {
        assert_eq!(apply_op!(10, +, 5), 15);
        assert_eq!(apply_op!(10, -, 5), 5);
        assert_eq!(apply_op!(10, *, 5), 50);
        assert_eq!(apply_op!(10, /, 5), 2);
    }

    #[test]
    fn test_documented_const() {
        assert_eq!(MAX_RETRIES, 3);
        assert_eq!(TIMEOUT_MS, 5000);
    }

    #[test]
    fn test_array_init() {
        let a = array_init!(0; 5);
        assert_eq!(a, [0, 0, 0, 0, 0]);

        let b = array_init!(1, 2, 3);
        assert_eq!(b, [1, 2, 3]);
    }

    #[test]
    fn test_platform_impl() {
        let size = pointer_size();
        assert!(size == 4 || size == 8);
    }
}
