# 手動メモリ管理

## 概要

Rustの所有権システムによる自動管理の代わりに、手動でメモリのライフタイムを制御します。

## Box::into_raw / Box::from_raw

```rust
// BoxをRaw pointerに変換（所有権放棄）
let boxed = Box::new(42_i32);
let raw: *mut i32 = Box::into_raw(boxed);

// 操作...
unsafe { *raw = 100; }

// 必ず回収してメモリを解放
let recovered = unsafe { Box::from_raw(raw) };
drop(recovered); // ここでメモリ解放
```

**注意**: `into_raw`後に`from_raw`しないとメモリリークになる。

## mem::forget

```rust
// dropを呼ばずに値を消す（リーク）
let v = vec![1, 2, 3];
let ptr = v.as_ptr();
mem::forget(v); // dropをスキップ、ptrは宙ぶらりん
```

## ManuallyDrop

```rust
use std::mem::ManuallyDrop;

let s = ManuallyDrop::new(String::from("hello"));
// スコープを出てもdropされない

// 明示的にdrop
let owned = ManuallyDrop::into_inner(s); // dropは通常通り
```

## ptr::read / ptr::write

```rust
// read: ポインタから値を「移動」（コピー）
// 元のメモリはまだ初期化済みだが所有権を持った側がdropする
let val: T = unsafe { ptr::read(src_ptr) };

// write: ポインタに値を書き込む（既存値をdropしない）
// 未初期化メモリへの最初の書き込みに使う
unsafe { ptr::write(dst_ptr, new_value) };
```

## ptr::drop_in_place

```rust
// ポインタが指す値のデストラクタを実行（メモリは解放しない）
unsafe { ptr::drop_in_place(ptr) };
// ptr以降のデメモリへのアクセスは未定義動作
```

## ptr::copy / ptr::copy_nonoverlapping

```rust
// memmove相当（重複可）
unsafe { ptr::copy(src, dst, count) };

// memcpy相当（重複不可、より高速）
unsafe { ptr::copy_nonoverlapping(src, dst, count) };
```

## グローバルアロケータ

```rust
use std::alloc::{alloc, dealloc, Layout};

let layout = Layout::new::<i32>();
let ptr = unsafe { alloc(layout) as *mut i32 };
assert!(!ptr.is_null());

unsafe {
    ptr::write(ptr, 42);
    // 使用...
    dealloc(ptr as *mut u8, layout);
}
```

## MaybeUninit

```rust
use std::mem::MaybeUninit;

// 未初期化メモリを安全に表現
let mut x = MaybeUninit::<i32>::uninit();
unsafe { x.as_mut_ptr().write(42) }; // または x.write(42)
let val = unsafe { x.assume_init() };
```

**廃止**: `mem::uninitialized()`は未定義動作を引き起こしやすいため削除済み。  
代わりに`MaybeUninit::uninit()`を使う。

## カスタムアロケータ

```rust
use std::alloc::{GlobalAlloc, Layout};

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { /* ... */ }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { /* ... */ }
}

#[global_allocator]
static A: MyAllocator = MyAllocator;
```

## 実装ファイル

`src/memory_management.rs`
