# 標準ライブラリの組み込みマクロ（Built-in Macros）

## カテゴリ一覧

| カテゴリ | マクロ |
|---|---|
| 出力 | `println!`, `print!`, `eprintln!`, `eprint!` |
| フォーマット | `format!`, `write!`, `writeln!`, `format_args!` |
| デバッグ | `dbg!`, `todo!`, `unimplemented!`, `unreachable!` |
| アサート | `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` |
| パニック | `panic!` |
| コレクション | `vec!` |
| 文字列 | `concat!`, `stringify!`, `include_str!`, `include_bytes!` |
| 環境 | `env!`, `option_env!`, `file!`, `line!`, `column!`, `module_path!` |
| コンパイル制御 | `cfg!`, `compile_error!` |
| その他 | `matches!` |

## フォーマット系

### `format!`

```rust
format!("{:>10}", "right")    // 右寄せ幅10
format!("{:<10}", "left")     // 左寄せ幅10
format!("{:^10}", "center")   // 中央寄せ幅10
format!("{:0>5}", 42)         // ゼロパディング → "00042"
format!("{:.3}", 3.14159)     // 小数点以下3桁 → "3.142"
format!("{:b}", 42)           // 2進数 → "101010"
format!("{:o}", 42)           // 8進数 → "52"
format!("{:x}", 255)          // 16進数(小文字) → "ff"
format!("{:X}", 255)          // 16進数(大文字) → "FF"
format!("{:#b}", 42)          // 0b付き2進数 → "0b101010"
format!("{:#x}", 255)         // 0x付き16進数 → "0xff"
format!("{:?}", vec![1,2,3])  // デバッグフォーマット
format!("{:#?}", (1,"hi"))    // pretty-print
format!("{name} is {age}", name="Alice", age=30) // 名前付き引数
format!("{0} {1} {0}", "a", "b")  // インデックス指定 → "a b a"
```

### `write!` / `writeln!`

```rust
use std::fmt::Write as FmtWrite;  // Stringに書き込む場合
use std::io::Write as IoWrite;    // Vec<u8>に書き込む場合

let mut s = String::new();
write!(s, "Hello, {}!", "world").unwrap();

let mut buf: Vec<u8> = Vec::new();
writeln!(&mut buf, "{}", 42).unwrap();
```

### `format_args!`

アロケーションなしでフォーマット引数を作る（ロギングなどで効率的）:

```rust
let args = format_args!("value: {}", 42);
let s = args.to_string(); // ここでアロケーション
```

## デバッグ系

### `dbg!`

値を標準エラーに表示しつつ、値をそのまま返す:

```rust
let x = 1;
let y = dbg!(x + 2);  // stderr: "[src/main.rs:2] x + 2 = 3"
assert_eq!(y, 3);     // 値は保持される
```

### `todo!` / `unimplemented!` / `unreachable!`

```rust
// todo!: 実装中のプレースホルダー（コンパイル通過、実行時パニック）
fn not_done() -> i32 {
    todo!("後で実装する")
}

// unimplemented!: サポートしない機能のマーカー
fn unsupported(op: &str) -> i32 {
    match op {
        "double" => 2,
        _ => unimplemented!("Operation '{}' not supported", op),
    }
}

// unreachable!: 論理的に到達不可能なコードのマーカー
fn direction(dir: &str) -> u32 {
    match dir {
        "N" => 0, "E" => 90, "S" => 180, "W" => 270,
        _ => unreachable!("Invalid direction"),
    }
}
```

## アサートマクロ

```rust
assert!(1 + 1 == 2);
assert!(true, "メッセージ付き");

assert_eq!(2 + 2, 4);
assert_eq!("hello".len(), 5, "カスタムメッセージ");

assert_ne!(1, 2);

// デバッグビルドのみ実行（リリースビルドでは無視）
debug_assert!(1 < 2);
debug_assert_eq!(3 * 3, 9);
debug_assert_ne!(0, 1);
```

## 文字列系

```rust
// concat!: コンパイル時文字列結合（&'static str）
const GREETING: &str = concat!("Hello", ", ", "Rust", "!");

// stringify!: トークンを文字列化
assert_eq!(stringify!(1 + 2), "1 + 2");
assert_eq!(stringify!(let x = 5), "let x = 5");

// include_str!: ファイルをコンパイル時に文字列として埋め込む
// const README: &str = include_str!("../README.md");

// include_bytes!: ファイルをバイト列として埋め込む
// const ICON: &[u8] = include_bytes!("icon.png");
```

## 環境・位置情報マクロ

```rust
// コンパイル時の情報
let file    = file!();         // ファイルパス（&'static str）
let line    = line!();         // 行番号（u32）
let col     = column!();       // 列番号（u32）
let module  = module_path!();  // モジュールパス（&'static str）

// 環境変数（コンパイル時取得）
// const HOME: &str = env!("HOME");  // 存在しないとコンパイルエラー
let var: Option<&'static str> = option_env!("OPTIONAL_VAR");  // Noneを返す
```

## `cfg!`

```rust
// 実行時に評価されるbool（#[cfg(...)]と異なりコードは常に残る）
let is_debug = cfg!(debug_assertions);
let is_linux = cfg!(target_os = "linux");
let is_64bit = cfg!(target_pointer_width = "64");

// 条件分岐での使用
let platform = if cfg!(target_os = "linux") { "linux" }
               else if cfg!(target_os = "macos") { "macos" }
               else { "other" };
```

## `matches!`

```rust
let v: Vec<Option<i32>> = vec![Some(1), None, Some(3)];

matches!(Some(42), Some(_))      // true
matches!(None::<i32>, Some(_))   // false
matches!(42_i32, 1..=50)         // true（範囲パターン）
matches!(42_i32, 1 | 42 | 100)  // true（orパターン）

// フィルタリングに便利
let count = v.iter().filter(|x| matches!(x, Some(_))).count();
```

## `vec!`の内部実装

```rust
macro_rules! vec {
    () => { Vec::new() };
    ($elem:expr; $n:expr) => {{
        let mut v = Vec::with_capacity($n);
        v.resize($n, $elem);
        v
    }};
    ($($x:expr),+ $(,)?) => {{
        // 実際の実装はBox経由でより効率的
        let mut v = Vec::with_capacity(/* count */);
        $(v.push($x);)+
        v
    }};
}
```

## 実装ファイル

`learning/src/builtin_macros.rs`
