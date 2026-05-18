//! # unsafeトレイト
//!
//! `unsafe trait`は「実装者が特定の不変条件を守ることを約束する」トレイトです。
//! 実装には`unsafe impl`が必要で、コンパイラではなく**人間**が正確性を保証します。
//!
//! ## 代表的なunsafeトレイト
//!
//! | トレイト | 不変条件 |
//! |---|---|
//! | `Send` | 型をスレッド間で転送しても安全 |
//! | `Sync` | 型へのimmutableな参照をスレッド間で共有しても安全 |
//! | `GlobalAlloc` | アロケータのAPIを正しく実装する |
//! | `Allocator` | アロケータAPIの詳細な契約を満たす |
//!
//! ## PhantomData
//!
//! `PhantomData<T>`はゼロサイズの幽霊型で、
//! 型パラメータの変位（variance）やドロップ（drop）の動作を制御します。

use std::marker::PhantomData;
use std::sync::Arc;

// ─── カスタムunsafeトレイトの定義 ──────────────────────────────────────────

/// バイト列に変換できる型のunsafeトレイト
///
/// 実装者はすべてのビットパターンが有効であることを保証しなければならない。
/// （例: 参照、bool、enumは実装不可）
pub unsafe trait PlainOldData {
    fn as_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    fn from_bytes(bytes: &[u8]) -> Option<&Self>
    where
        Self: Sized,
    {
        if bytes.len() < std::mem::size_of::<Self>() {
            return None;
        }
        // SAFETY: 実装者がすべてのビットパターンの有効性を保証
        Some(unsafe { &*(bytes.as_ptr() as *const Self) })
    }
}

/// すべてのビットパターンが有効なi32（POD）
// SAFETY: i32はすべてのビットパターンが有効な値として定義される
unsafe impl PlainOldData for i32 {}

/// すべてのビットパターンが有効なu8（POD）
// SAFETY: u8はすべてのビットパターンが有効
unsafe impl PlainOldData for u8 {}

/// 複合型のPOD実装
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// SAFETY: f32はすべてのビットパターン（NaN含む）を受け入れ、
// Vector3は#[repr(C)]でパディングが制御される
unsafe impl PlainOldData for Vector3 {}

// ─── Send / Sync の手動実装 ────────────────────────────────────────────────

/// 生ポインタを格納する型（通常はSend/Syncを自動実装しない）
pub struct MyBox<T> {
    ptr: *mut T,
    _phantom: PhantomData<T>,
}

impl<T> MyBox<T> {
    pub fn new(val: T) -> Self {
        let ptr = Box::into_raw(Box::new(val));
        MyBox {
            ptr,
            _phantom: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        // SAFETY: ptrは有効なTを指す（constructorで設定）
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: ptrは有効なTを指し、MyBoxが所有権を持つ
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        // SAFETY: ptrはMyBox::newで設定、まだ解放されていない
        unsafe { drop(Box::from_raw(self.ptr)) }
    }
}

// SAFETY: MyBoxはTを排他的に所有し、スレッド間での転送が安全
unsafe impl<T: Send> Send for MyBox<T> {}

// SAFETY: MyBoxへの参照から&Tを得られるが、TがSyncなら安全
unsafe impl<T: Sync> Sync for MyBox<T> {}

// ─── Syncでない型のラッパー ─────────────────────────────────────────────────

/// スレッドローカルな値を示すラッパー（Syncではない）
///
/// `!Sync`にするために`PhantomData<*const ()>`を使う。
/// 生ポインタはSend/Syncを実装しないため。
pub struct ThreadLocal<T> {
    val: T,
    _not_sync: PhantomData<*const ()>,
}

impl<T> ThreadLocal<T> {
    pub fn new(val: T) -> Self {
        ThreadLocal {
            val,
            _not_sync: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        &self.val
    }
}

// ThreadLocalはSendだが（所有権の移転は安全）、Syncではない
// SAFETY: 所有権の移転は安全
unsafe impl<T: Send> Send for ThreadLocal<T> {}
// Syncは意図的に実装しない（*const ()のせいで自動実装されない）

// ─── PhantomData の変位（Variance）制御 ─────────────────────────────────────

/// 共変（covariant）なスマートポインタ
///
/// PhantomData<T>は&Tと同じ変位: 'b: 'a なら MyRef<'a, T>: MyRef<'b, T>
pub struct MyRef<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> MyRef<'a, T> {
    pub fn new(val: &'a T) -> Self {
        MyRef {
            ptr: val as *const T,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        // SAFETY: 'aの生存期間内なのでptrは有効
        unsafe { &*self.ptr }
    }
}

/// 反変（contravariant）なラッパー
///
/// PhantomData<fn(T)>はfn(T)と同じ変位（反変）
pub struct Sink<T> {
    _marker: PhantomData<fn(T)>,
}

impl<T> Sink<T> {
    pub fn new() -> Self {
        Sink {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for Sink<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 不変（invariant）なラッパー
///
/// PhantomData<fn(T) -> T>はinvariantになる
pub struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,
}

impl<T> Invariant<T> {
    pub fn new() -> Self {
        Invariant {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for Invariant<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Drop の安全性と PhantomData ─────────────────────────────────────────────

/// DropcheckのためにPhantomDataを使う型
///
/// Tへの参照を持つが生ポインタで格納する場合、
/// PhantomData<T>でdropcheckに参加させる。
pub struct OwnedPtr<T> {
    ptr: *mut T,
    _owns: PhantomData<T>, // "TはOwnedPtrがdropされる際に生きている必要はない"の逆の宣言
}

impl<T> OwnedPtr<T> {
    pub fn new(val: T) -> Self {
        OwnedPtr {
            ptr: Box::into_raw(Box::new(val)),
            _owns: PhantomData,
        }
    }

    pub fn take(self) -> T {
        // SAFETY: ptrはnewで設定、まだ解放されていない
        let val = unsafe { std::ptr::read(self.ptr) };
        // dropで二重解放しないようにManuallyDropを使う
        let me = std::mem::ManuallyDrop::new(self);
        unsafe { std::alloc::dealloc(me.ptr as *mut u8, std::alloc::Layout::new::<T>()) };
        val
    }
}

impl<T> Drop for OwnedPtr<T> {
    fn drop(&mut self) {
        // SAFETY: ptrは有効、まだ解放されていない
        unsafe { drop(Box::from_raw(self.ptr)) }
    }
}

// ─── Arcを使うunsafeトレイトの実際の活用例 ──────────────────────────────────

/// Arcで共有する際にSyncが必要
pub fn share_across_threads<T: Send + Sync + 'static>(val: T) -> Arc<T> {
    Arc::new(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_i32_as_bytes() {
        let x: i32 = 0x01020304;
        let bytes = x.as_bytes();
        assert_eq!(bytes.len(), 4);
        // リトルエンディアン
        #[cfg(target_endian = "little")]
        assert_eq!(bytes[0], 0x04);
    }

    #[test]
    fn test_pod_from_bytes() {
        let bytes: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
        let val = i32::from_bytes(&bytes).unwrap();
        #[cfg(target_endian = "little")]
        assert_eq!(*val, 1);
    }

    #[test]
    fn test_pod_vector3() {
        let v = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let bytes = v.as_bytes();
        assert_eq!(bytes.len(), 12); // 3 * 4 bytes
    }

    #[test]
    fn test_my_box() {
        let mut b = MyBox::new(42_i32);
        assert_eq!(*b.get(), 42);
        *b.get_mut() = 100;
        assert_eq!(*b.get(), 100);
    }

    #[test]
    fn test_my_box_send() {
        let b = MyBox::new(String::from("hello"));
        let handle = std::thread::spawn(move || {
            assert_eq!(b.get(), "hello");
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_my_box_string() {
        let b = MyBox::new(String::from("test"));
        assert_eq!(b.get(), "test");
    }

    #[test]
    fn test_thread_local() {
        let tl = ThreadLocal::new(42_i32);
        assert_eq!(*tl.get(), 42);
    }

    #[test]
    fn test_my_ref() {
        let x = 99_i32;
        let r = MyRef::new(&x);
        assert_eq!(*r.get(), 99);
    }

    #[test]
    fn test_owned_ptr() {
        let op = OwnedPtr::new(String::from("owned"));
        let s = op.take();
        assert_eq!(s, "owned");
    }

    #[test]
    fn test_share_across_threads() {
        let arc = share_across_threads(42_i32);
        let arc2 = Arc::clone(&arc);
        let handle = std::thread::spawn(move || {
            assert_eq!(*arc2, 42);
        });
        handle.join().unwrap();
        assert_eq!(*arc, 42);
    }

    #[test]
    fn test_pod_from_bytes_too_short() {
        let bytes = [0_u8; 2]; // i32には4バイト必要
        assert!(i32::from_bytes(&bytes).is_none());
    }
}
