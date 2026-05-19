# 属性マクロ（Procedural Attribute Macros）

## 概要

`#[attr]`形式でアイテム（関数、構造体など）を変換する手続きマクロです。アイテム全体を受け取り、変換後のコードを返します。

## 実装の仕組み

```rust
#[proc_macro_attribute]
pub fn my_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr: #[my_attr(ここの内容)]
    // item: 属性が付いたアイテム全体
    // 戻り値: 変換後のコード（元のアイテムの代わりに展開される）
}
```

deriveマクロと異なり、**元のアイテムはそのまま残らない**。出力に含めなければ消える。

## このプロジェクトの属性マクロ

### `#[logged]`

関数の入口と出口をログ出力する:

```rust
#[logged]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

let result = add(3, 4);
// stderr: "[CALL] add"
// stderr: "[RETURN] add -> 0x..."
assert_eq!(result, 7);
```

**実装**: 関数ボディをクロージャでラップし、前後にprintln!を挿入:

```rust
#vis #sig {
    println!("[CALL] {}", #fname_str);
    let __result = (|| #block)();
    println!("[RETURN] {} -> {:?}", #fname_str, &__result as *const _);
    __result
}
```

### `#[retry(times = N)]`

関数が`Err`を返した場合にN回リトライする:

```rust
use std::sync::atomic::{AtomicI32, Ordering};
static COUNTER: AtomicI32 = AtomicI32::new(0);

#[retry(times = 3)]
fn might_fail() -> Result<i32, String> {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    if n < 2 { Err("not yet".to_string()) } else { Ok(42) }
}

assert_eq!(might_fail(), Ok(42));  // 2回失敗して3回目に成功
```

**属性引数のパース**（簡易版）:

```rust
let times: usize = {
    let attr_str = attr.to_string();
    if attr_str.contains("times") {
        attr_str.split('=').nth(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(3)
    } else {
        3
    }
};
```

より堅牢なパースには`syn::parse`とカスタム構造体を使う。

**生成されるコード**:

```rust
fn might_fail() -> Result<i32, String> {
    let mut __attempts = 0;
    loop {
        let __result = (|| { /* 元のbody */ })();
        __attempts += 1;
        match __result {
            Ok(v) => return Ok(v),
            Err(e) if __attempts < 3 => {
                eprintln!("[RETRY] might_fail failed (attempt {}): {:?}", __attempts, e);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### `#[measure_time]`

関数の実行時間を計測する:

```rust
#[measure_time]
fn slow_operation() -> i32 {
    42
}

let result = slow_operation();
// stderr: "[TIME] slow_operation took 123ns"
assert_eq!(result, 42);
```

**生成されるコード**:

```rust
fn slow_operation() -> i32 {
    let __start = std::time::Instant::now();
    let __result = (|| { 42 })();
    let __elapsed = __start.elapsed();
    eprintln!("[TIME] {} took {:?}", "slow_operation", __elapsed);
    __result
}
```

## 属性マクロの実装パターン

### 関数変換の基本構造

```rust
#[proc_macro_attribute]
pub fn my_wrapper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let fname  = &func.sig.ident;
    let block  = &func.block;
    let sig    = &func.sig;
    let vis    = &func.vis;
    let attrs  = &func.attrs;  // 他の属性を保持

    let expanded = quote! {
        #(#attrs)*     // 既存の属性を保持
        #vis #sig {
            // 前処理
            let __result = (|| #block)();  // 元のbodyをクロージャでラップ
            // 後処理
            __result
        }
    };
    TokenStream::from(expanded)
}
```

### 属性引数の取り扱い

```rust
// #[my_attr]          → attr = ""
// #[my_attr(a = 1)]   → attr = "a = 1"
// #[my_attr("str")]   → attr = "\"str\""

#[proc_macro_attribute]
pub fn my_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 文字列パース（簡易）
    let attr_str = attr.to_string();

    // syn でパース（厳密）
    // let args = syn::parse::<MyArgs>(attr).unwrap();

    // ...
}
```

## deriveマクロとの違い

| 特性 | derive | attribute |
|---|---|---|
| 元のアイテム | **保持される**（追加のみ） | **置き換わる**（出力に含めないと消える） |
| 適用対象 | struct, enum | 任意のアイテム |
| 引数 | ヘルパー属性のみ | `#[attr(args)]`で渡せる |
| 典型的用途 | トレイト実装の追加 | 関数のラッピング、コード変換 |

## 実装ファイル

- マクロ定義: `macros_impl/src/lib.rs`
- 使用例: `learning/src/proc_attribute.rs`
