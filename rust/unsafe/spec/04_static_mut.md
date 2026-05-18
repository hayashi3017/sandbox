# 可変静的変数（`static mut`）

## 概要

グローバルスコープの可変変数。マルチスレッド環境ではデータ競合の危険があります。

## 基本構文

```rust
static mut COUNTER: i32 = 0;

// アクセスはunsafe
unsafe {
    COUNTER += 1;
    println!("{}", COUNTER);
}
```

## データ競合の危険性

```rust
// 危険: マルチスレッドで同時アクセスするとデータ競合
static mut FLAG: bool = false;

thread::spawn(|| unsafe { FLAG = true; });
unsafe { println!("{}", FLAG); } // 未定義動作
```

## 安全な代替手段

### AtomicXxx（整数・ブール値）

```rust
use std::sync::atomic::{AtomicI32, Ordering};

static COUNTER: AtomicI32 = AtomicI32::new(0);

COUNTER.fetch_add(1, Ordering::SeqCst);  // スレッドセーフ
let val = COUNTER.load(Ordering::SeqCst);
```

### Mutex（任意の型）

```rust
use std::sync::Mutex;

static DATA: Mutex<Vec<String>> = Mutex::new(Vec::new());

DATA.lock().unwrap().push("hello".to_string());
```

### OnceLock（遅延初期化）

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

let val = CONFIG.get_or_init(|| "default".to_string());
```

## Rust 2024 editionでの変更

Rust 2024 editionでは`static mut`への参照取得がhard errorになります:

```rust
// Rust 2024では error[E0796]
static mut X: i32 = 0;
let r = unsafe { &X }; // エラー
```

代わりに`addr_of!`マクロや`AtomicPtr`を使います。

## Ordering（メモリ順序）の選択指針

| Ordering | 用途 |
|---|---|
| `Relaxed` | カウンタ（順序不要） |
| `Acquire`/`Release` | ロックの取得/解放 |
| `SeqCst` | グローバルな全順序が必要な場合 |

## 実装ファイル

`src/static_mut.rs`
