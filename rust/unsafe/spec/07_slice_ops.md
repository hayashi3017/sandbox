# 生ポインタからのスライス/文字列生成

## 概要

`slice::from_raw_parts`等で、生ポインタと長さからスライスや文字列を生成します。  
主にFFIバウンダリやカスタムコレクションの実装で使います。

## 主なAPI

```rust
use std::slice;
use std::str;

// 不変スライス生成
let s: &[T] = unsafe { slice::from_raw_parts(ptr, len) };

// 可変スライス生成
let s: &mut [T] = unsafe { slice::from_raw_parts_mut(ptr, len) };

// UTF-8検証なしで&strへ
let s: &str = unsafe { str::from_utf8_unchecked(bytes) };
```

## 安全条件（from_raw_parts）

1. `ptr`は非null
2. `ptr`は`T`の適切なアライメントを満たす
3. `ptr`から`len * size_of::<T>()`バイトのメモリが有効
4. そのメモリは有効に初期化されている
5. `len <= isize::MAX`
6. 返り値の生存期間中、メモリは変更されない（不変の場合）

## FFIパターン

```rust
// Cから渡されたバッファをRustのスライスとして使う
extern "C" fn process_data(buf: *const u8, len: usize) {
    // SAFETY: Cが有効なbuf/lenを渡すことを保証
    let data = unsafe { slice::from_raw_parts(buf, len) };
    // dataをRustのスライスとして使う
}
```

## 2つの可変参照を同一スライスから取得

```rust
// 借用チェッカーでは不可だが、インデックスが異なれば安全
fn split_two_mut<T>(s: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j);
    let ptr = s.as_mut_ptr();
    unsafe {
        let a = &mut *ptr.add(i);
        let b = &mut *ptr.add(j);
        (a, b) // i != jなのでエイリアスなし
    }
}
```

## align_to

```rust
// u8スライスをu32のアライメント境界で3分割
let (prefix, aligned, suffix): (&[u8], &[u32], &[u8]) = unsafe {
    bytes.align_to::<u32>()
};
// prefixとsuffixはアライメント調整用の端数バイト
```

## str::from_utf8_unchecked の注意点

```rust
// safe版（推奨）
let s = str::from_utf8(bytes)?;

// unsafe版（UTF-8であることが確実な場合のみ）
let s = unsafe { str::from_utf8_unchecked(bytes) };
// 無効なUTF-8を渡すと後続の文字列操作が未定義動作になる
```

## 実装ファイル

`src/slice_ops.rs`
