# 生ポインタ（Raw Pointers）

## 概要

生ポインタはRustの参照とは異なり、借用チェッカーの管理外にあるポインタです。

## 型

| 型 | 説明 |
|---|---|
| `*const T` | 不変生ポインタ（値の変更不可） |
| `*mut T` | 可変生ポインタ（値の変更可） |

## 参照との比較

| 特性 | `&T` / `&mut T` | `*const T` / `*mut T` |
|---|---|---|
| null許容 | 不可（常にnon-null） | 可能 |
| 借用チェック | コンパイル時に検証 | なし |
| 生存期間追跡 | あり | なし |
| 自動deref | あり | なし（unsafeブロック必要） |
| `Send`/`Sync` | Tに従う | 実装しない |

## 生成方法

```rust
// 参照からキャスト（最も一般的）
let x = 42_i32;
let const_ptr: *const i32 = &x as *const i32;
let const_ptr2: *const i32 = &x; // 自動キャスト

let mut y = 42_i32;
let mut_ptr: *mut i32 = &mut y as *mut i32;
let mut_ptr2: *mut i32 = &mut y; // 自動キャスト

// null生成
let null: *const i32 = std::ptr::null();
let null_mut: *mut i32 = std::ptr::null_mut();

// アドレスから生成（危険）
let addr: usize = 0x12345678;
let dangerous: *const i32 = addr as *const i32;
```

## 主なメソッド

| メソッド | 型 | 説明 |
|---|---|---|
| `ptr.is_null()` | `bool` | nullチェック |
| `*ptr` | `T` | deref（unsafe） |
| `ptr.as_ref()` | `Option<&T>` | 安全な参照変換（unsafe） |
| `ptr.as_mut()` | `Option<&mut T>` | 安全な可変参照変換（unsafe） |
| `ptr.read()` | `T` | コピー（unsafe） |
| `ptr.write(val)` | `()` | 書き込み（unsafe） |
| `ptr.cast::<U>()` | `*const U` | 型キャスト |
| `ptr as usize` | `usize` | アドレスの数値変換 |

## NonNull\<T\>

```rust
use std::ptr::NonNull;

let mut x = 42;
let nn: NonNull<i32> = NonNull::new(&mut x as *mut i32).unwrap();

// Option<NonNull<T>>はOption<Box<T>>と同じサイズ（ニッチ最適化）
assert_eq!(
    std::mem::size_of::<Option<NonNull<i32>>>(),
    std::mem::size_of::<*mut i32>()
);
```

## 安全条件

生ポインタをdereferenceする際は以下を満たす必要があります:

1. ポインタはnullでない
2. ポインタは適切にアライメントされている
3. ポインタが指すメモリは有効に初期化されている
4. アクセス期間中、メモリは有効である（解放されていない）
5. 可変参照の場合、他の参照が存在しない（エイリアスがない）

## 実装ファイル

`src/raw_pointers.rs`
