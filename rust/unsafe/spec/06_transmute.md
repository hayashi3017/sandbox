# `mem::transmute` — メモリの再解釈

## 概要

`transmute<T, U>(val: T) -> U`はビットパターンを変えずに型を変換します。  
コンパイル時に`size_of::<T>() == size_of::<U>()`が検証されます。

## 基本

```rust
use std::mem;

// f32のビットをu32として読む
let f: f32 = 1.0;
let bits: u32 = unsafe { mem::transmute(f) };
assert_eq!(bits, 0x3F800000);
```

## よくある用途と安全な代替

| 操作 | transmute | 安全な代替 |
|---|---|---|
| f32 → u32（ビット） | `transmute(f)` | `f.to_bits()` |
| u32 → f32（ビット） | `transmute(u)` | `f32::from_bits(u)` |
| i32 → u32 | `transmute(i)` | `i as u32` |
| `&T` → `&U`（同サイズ） | `transmute(r)` | `ptr::cast()` |
| 配列 → スライス | `transmute(&arr)` | `arr.as_slice()` |

## 危険な用途

### ❌ 生存期間の拡張

```rust
// 極めて危険: use-after-freeの可能性
unsafe fn extend_lifetime<T>(r: &T) -> &'static T {
    std::mem::transmute(r)
}
```

### ❌ 無効なビットパターン

```rust
// 未定義動作: boolは0か1のみ有効
let invalid_bool: bool = unsafe { mem::transmute(2_u8) };
```

### ❌ アライメント不一致

```rust
// 未定義動作: u8のアドレスはu32のアライメント要件を満たさない場合がある
let bytes = [0u8; 4];
let _: &u32 = unsafe { mem::transmute(&bytes[1]) };
```

## transmute_copy

```rust
// サイズチェックなし（コピーしてから変換）
let x: u32 = 0x01020304;
let bytes: [u8; 4] = unsafe { mem::transmute_copy(&x) };
```

## MaybeUninit（transmute代替）

```rust
// transmute::<[MaybeUninit<T>; N], [T; N]>の代わりに
let arr: [u32; 3] = [1, 2, 3];
// transmute不要で配列を扱える場合が多い
```

## 検証ツール

```bash
# Miriで未定義動作を検出
cargo +nightly miri test
```

## 実装ファイル

`src/transmute_ops.rs`
