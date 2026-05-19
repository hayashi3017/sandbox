# 再帰マクロとTTマンチャー（Recursive Macros & TT Muncher）

## 概要

マクロは自分自身を再帰的に呼び出せます。この性質を使ってリストの処理や複雑なDSLパーサーを実装できます。

## 基本パターン

```text
macro_rules! my_macro {
    ()                      => { /* ベースケース */ };
    ($head:tt $($tail:tt)*) => {
        /* headを処理し、tailを再帰 */
        my_macro!($($tail)*)
    };
}
```

**重要**: 必ずベースケースを用意し、各再帰ステップで少なくとも1トークンを消費すること。そうしないと無限再帰になる。

## TTマンチャー（Token Tree Muncher）

入力トークン列を先頭から「かじって」処理していくパターン。

```rust
// トークン個数をカウント
macro_rules! count_tts {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => {
        1usize + count_tts!($($tail)*)
    };
}

assert_eq!(count_tts!(a b c), 3);

// カスタム算術DSL
macro_rules! calc {
    ($x:literal) => { $x };
    ($x:literal + $($rest:tt)+) => { $x + calc!($($rest)+) };
    ($x:literal * $($rest:tt)+) => { $x * calc!($($rest)+) };
    ($x:literal - $($rest:tt)+) => { $x - calc!($($rest)+) };
}

assert_eq!(calc!(1 + 2 + 3), 6);
```

## Push-down Accumulation

結果を「蓄積引数」に積み上げていく再帰パターン。Rustのマクロは左再帰が使えないためこのパターンが重要。

```rust
// リストを逆順にする
macro_rules! reverse_list {
    // 内部ルール: ベースケース
    (@rev [$($acc:expr),*]) => {
        [$($acc),*]
    };
    // 内部ルール: 再帰（先頭を蓄積の先頭に積む）
    (@rev [$($acc:expr),*] $head:expr $(, $tail:expr)*) => {
        reverse_list!(@rev [$head $(, $acc)*] $($tail),*)
    };
    // 公開インターフェース
    ($($xs:expr),*) => {
        reverse_list!(@rev [] $($xs),*)
    };
}

let r = reverse_list![1, 2, 3, 4, 5];
assert_eq!(r, [5, 4, 3, 2, 1]);
```

### なぜ`@rev`で分離するのか

`@`なしで`reverse_list!([] 1, 2, 3)`と呼び出すと、`[$($xs:expr),*]`アームが`[]`を単一の配列式としてマッチし、再帰が止まらなくなる。`@rev`プレフィックスで内部ルールを明示的に分離することで、外部呼び出しと内部再帰を区別する。

## リスト操作マクロ

```rust
// 先頭要素を取り出す
macro_rules! head {
    ($head:expr $(, $_tail:expr)*) => { $head };
}

// 末尾要素を取り出す
macro_rules! tail {
    ($only:expr) => { $only };
    ($_head:expr, $($rest:expr),+) => { tail!($($rest),+) };
}

assert_eq!(head!(10, 20, 30), 10);
assert_eq!(tail!(10, 20, 30), 30);
```

## 再帰的な算術

```rust
macro_rules! sum_recursive {
    () => { 0 };
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        $x + sum_recursive!($($rest),+)
    };
}

macro_rules! max_recursive {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let rest = max_recursive!($($rest),+);
        if $x > rest { $x } else { rest }
    }};
}

assert_eq!(sum_recursive!(1, 2, 3, 4, 5), 15);
assert_eq!(max_recursive!(1, 5, 3, 2, 4), 5);
```

## コールバックパターン（高階マクロ）

マクロに別のマクロを渡すパターン:

```rust
macro_rules! apply_to_each {
    ($callback:ident![$($items:expr),+]) => {
        vec![$($callback!($items)),+]
    };
}

macro_rules! square_item {
    ($x:expr) => { $x * $x };
}

let squares = apply_to_each!(square_item![2, 3, 4]);
assert_eq!(squares, vec![4, 9, 16]);
```

## テストケース自動生成

```rust
macro_rules! test_cases {
    ($func:ident: $($input:expr => $expected:expr),+ $(,)?) => {
        $(
            {
                let result = $func($input);
                assert_eq!(result, $expected,
                    "{}({:?}) expected {:?}",
                    stringify!($func), $input, $expected);
            }
        )+
    };
}

// テスト内で使用:
test_cases! {
    double_fn:
    0  => 0,
    1  => 2,
    5  => 10,
}
```

## 注意事項

- **相互再帰とトークン評価**: マクロはトークンを操作するため、`2 - 1`は`1`に評価されない。リテラル`1`と`2 - 1`は別のトークン列。
- **再帰深度の制限**: デフォルトは128。`#![recursion_limit = "256"]`で増やせる。
- **フィルタのexpr制限**: `expr`フラグメントの後続は`=>`, `,`, `;`のみ。DSLのキーワードを続けるにはカッコで囲む: `filter!((v) WHERE ...)`

## 実装ファイル

`learning/src/recursive.rs`
