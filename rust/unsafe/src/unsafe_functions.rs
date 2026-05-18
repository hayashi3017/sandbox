//! # unsafe関数の定義と呼び出し
//!
//! `unsafe fn`は「呼び出し元が特定の不変条件を守る責任を持つ」ことを示す契約です。
//!
//! ## unsafeブロックの5つの用途
//!
//! 1. 生ポインタのderef
//! 2. unsafe関数/メソッドの呼び出し
//! 3. 可変静的変数へのアクセス
//! 4. unsafeトレイトの実装
//! 5. `union`フィールドへのアクセス
//!
//! ## 設計指針
//!
//! - `unsafe fn`には必ず`# Safety`セクションのドキュメントを付ける
//! - unsafe操作を薄くラップして安全なAPIを提供する（safety abstraction）
//! - unsafeブロックは最小限の範囲に留める

use std::ptr;

/// # Safety
///
/// `ptr`は有効なi32を指し、呼び出し期間中有効でなければならない。
pub unsafe fn deref_raw(ptr: *const i32) -> i32 {
    *ptr
}

/// # Safety
///
/// `ptr`は有効なi32を指し、呼び出し期間中排他的に書き込み可能でなければならない。
pub unsafe fn write_raw(ptr: *mut i32, val: i32) {
    *ptr = val;
}

/// unsafeな操作を安全なインターフェースで包む（safety abstraction）
///
/// 内部ではunsafeを使うが、公開APIは安全。
/// このパターンはRustの標準ライブラリ全体で使われている。
pub fn get_first(slice: &[i32]) -> Option<i32> {
    if slice.is_empty() {
        return None;
    }
    let ptr: *const i32 = slice.as_ptr();
    // SAFETY: スライスが非空なので先頭要素は有効
    Some(unsafe { *ptr })
}

/// unsafeブロックの最小化の例
///
/// unsafeブロックは必要な操作のみを含み、
/// 通常のRustコードはブロックの外に置く。
pub fn sum_via_pointer(data: &[i32]) -> i32 {
    let mut sum = 0_i32;
    let mut ptr = data.as_ptr();
    let end = unsafe { ptr.add(data.len()) };
    while ptr != end {
        // SAFETY: ptr は [data.as_ptr(), end) の範囲内
        sum += unsafe { *ptr };
        // SAFETY: 同上
        ptr = unsafe { ptr.add(1) };
    }
    sum
}

/// 複数のunsafe操作を1つのunsafeブロックにまとめてよい場合
///
/// 同じ安全条件を共有する操作はまとめてunsafeブロックに書ける。
/// ただし、それぞれの操作の安全条件をコメントで明示する。
pub fn swap_raw(a: *mut i32, b: *mut i32) {
    // SAFETY:
    // - a, bは両方とも有効なi32を指す
    // - a != b（エイリアスなし）
    // - 呼び出し元がこれを保証する
    unsafe {
        let tmp = *a;
        *a = *b;
        *b = tmp;
    }
}

/// # Safety
///
/// `src`と`dst`は重複せず、両方とも`count`個のi32が書き込み/読み取り可能。
pub unsafe fn copy_nonoverlapping_i32(src: *const i32, dst: *mut i32, count: usize) {
    ptr::copy_nonoverlapping(src, dst, count);
}

/// unsafeなクロージャを受け取る関数
///
/// 引数にunsafeなFnを受け取るにはunsafe fn自体にする必要がある。
/// （Rustではunsafe closureの型はstable APIには直接現れない）
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

/// unsafeコードのインライン化を制御するヒント
///
/// `#[inline]`はコンパイラへのヒント。unsafeコードのパフォーマンス最適化に使う。
#[inline]
pub fn fast_copy_byte(src: *const u8, dst: *mut u8) {
    // SAFETY: src/dstは有効な1バイトを指す（呼び出し元保証）
    unsafe {
        *dst = *src;
    }
}

/// 関数ポインタのunsafe呼び出し
///
/// 関数ポインタ自体のderefはsafeだが、
/// `extern "C"` 関数ポインタの呼び出しはunsafe。
pub fn call_fn_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref_raw() {
        let x = 42_i32;
        let result = unsafe { deref_raw(&x as *const i32) };
        assert_eq!(result, 42);
    }

    #[test]
    fn test_write_raw() {
        let mut x = 0_i32;
        unsafe { write_raw(&mut x as *mut i32, 77) };
        assert_eq!(x, 77);
    }

    #[test]
    fn test_get_first() {
        assert_eq!(get_first(&[10, 20, 30]), Some(10));
        assert_eq!(get_first(&[]), None);
    }

    #[test]
    fn test_sum_via_pointer() {
        let data = [1, 2, 3, 4, 5];
        assert_eq!(sum_via_pointer(&data), 15);
        assert_eq!(sum_via_pointer(&[]), 0);
    }

    #[test]
    fn test_swap_raw() {
        let mut a = 10_i32;
        let mut b = 20_i32;
        swap_raw(&mut a as *mut i32, &mut b as *mut i32);
        assert_eq!(a, 20);
        assert_eq!(b, 10);
    }

    #[test]
    fn test_copy_nonoverlapping() {
        let src = [1_i32, 2, 3];
        let mut dst = [0_i32; 3];
        unsafe {
            copy_nonoverlapping_i32(src.as_ptr(), dst.as_mut_ptr(), 3);
        }
        assert_eq!(dst, [1, 2, 3]);
    }

    #[test]
    fn test_apply_twice() {
        assert_eq!(apply_twice(|x| x * 2, 3), 12);
    }

    #[test]
    fn test_fn_ptr() {
        fn double(x: i32) -> i32 {
            x * 2
        }
        assert_eq!(call_fn_ptr(double, 5), 10);
    }
}
