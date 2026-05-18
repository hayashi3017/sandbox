# unsafeトレイト

## 概要

`unsafe trait`は「実装者が特定の不変条件を守る」ことを約束するトレイトです。  
コンパイラでは検証できない契約をプログラマが保証します。

## 構文

```rust
// 定義
unsafe trait Foo {
    fn method(&self);
}

// 実装
unsafe impl Foo for MyType {
    fn method(&self) { /* ... */ }
}
```

## Send / Sync

Rustで最も重要なunsafeトレイトは`Send`と`Sync`です。

```rust
// Sendの手動実装（型をスレッド間で転送しても安全）
unsafe impl<T: Send> Send for MyBox<T> {}

// Syncの手動実装（型へのimmutable参照をスレッド間で共有しても安全）
unsafe impl<T: Sync> Sync for MyBox<T> {}
```

### 自動実装のルール

| 条件 | Send/Sync |
|---|---|
| 全フィールドがSend | Sendを自動実装 |
| 全フィールドがSync | Syncを自動実装 |
| `*const T`/`*mut T`を含む | Sendでも Syncでもない |
| `Cell<T>`を含む | Syncではない |
| `Rc<T>`を含む | Sendでも Syncでもない |

## !Send / !Sync にする方法

```rust
use std::marker::PhantomData;

// *const ()はSendもSyncも実装しない
struct NotSync {
    _not_sync: PhantomData<*const ()>,
}
// NotSyncはSendだがSyncではない（スレッドローカル用途）
```

## PhantomData による変位（Variance）制御

```rust
// 共変（covariant）: &Tと同じ変位
struct MyRef<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

// 反変（contravariant）: fn(T)と同じ変位
struct Sink<T> {
    _marker: PhantomData<fn(T)>,
}

// 不変（invariant）: fn(T) -> Tと同じ
struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,
}
```

## カスタムunsafeトレイトの設計

```rust
/// すべてのビットパターンが有効な型
///
/// # Safety
///
/// 実装者はすべての`u8`パターンが有効なSelf値を表すことを保証する。
/// 参照型、bool、enumは実装できない。
pub unsafe trait PlainOldData: Sized {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

// i32はすべてのビットパターンが有効
unsafe impl PlainOldData for i32 {}
```

## GlobalAlloc

```rust
use std::alloc::{GlobalAlloc, Layout};

unsafe trait GlobalAlloc {
    // SAFETY: layoutはsize > 0、アロケータの要件を満たす
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
    // ... デフォルト実装あり
}
```

## 実装ファイル

`src/unsafe_traits.rs`
