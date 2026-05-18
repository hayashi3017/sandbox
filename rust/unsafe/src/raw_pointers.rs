//! # 生ポインタ（Raw Pointers）
//!
//! Rustの生ポインタは `*const T`（不変）と `*mut T`（可変）の2種類があります。
//!
//! ## 参照との違い
//!
//! | 機能 | 参照 | 生ポインタ |
//! |---|---|---|
//! | null許可 | 不可 | 可 |
//! | 借用チェック | あり | なし |
//! | 自動deref | あり | なし（unsafeブロック必要） |
//! | 生存期間 | 追跡 | 追跡なし |
//!
//! ## 主なAPI
//! - `ptr::null::<T>()` / `ptr::null_mut::<T>()` — nullポインタ生成
//! - `ptr.is_null()` — nullチェック
//! - `*ptr` — deref（unsafe）
//! - `ptr.as_ref()` — `Option<&T>`への変換（unsafe）
//! - `ptr.as_mut()` — `Option<&mut T>`への変換（unsafe）
//! - `std::ptr::NonNull<T>` — nullでない保証付きポインタ

use std::ptr::{self, NonNull};

/// 参照から生ポインタを生成し、値を読み取る基本例
pub fn read_via_raw_pointer(value: &i32) -> i32 {
    // 参照から不変生ポインタへキャスト（unsafeブロック不要）
    let raw: *const i32 = value as *const i32;
    // dereferenceにはunsafeブロックが必要
    unsafe { *raw }
}

/// 可変生ポインタを介して値を書き換える
pub fn write_via_raw_pointer(value: &mut i32, new_val: i32) {
    let raw: *mut i32 = value as *mut i32;
    unsafe {
        *raw = new_val;
    }
}

/// nullポインタの生成と検査
pub fn demonstrate_null_pointer() -> bool {
    let null_ptr: *const i32 = ptr::null();
    null_ptr.is_null()
}

/// nullでないポインタの安全な参照変換（as_ref）
///
/// `as_ref()`はポインタがnullの場合`None`を返すため、
/// null参照を生成しないという点で`&*ptr`より安全。
pub fn safe_deref(ptr: *const i32) -> Option<i32> {
    // SAFETY: 呼び出し元がptrの有効性を保証する前提
    unsafe { ptr.as_ref().copied() }
}

/// 2つの生ポインタが同じアドレスを指すかチェック
pub fn pointers_are_equal(a: *const i32, b: *const i32) -> bool {
    a == b
}

/// NonNull<T>: nullでないことが保証された生ポインタのラッパー
///
/// `Option<NonNull<T>>`のサイズは`Option<Box<T>>`と同じ（ニッチ最適化）。
pub fn demonstrate_non_null(value: &mut i32) -> i32 {
    // NonNullはptrが確実にnullでない場合に使う
    let nn: NonNull<i32> = NonNull::from(value);
    // SAFETY: nnはvalueから作ったので有効
    unsafe { *nn.as_ptr() }
}

/// ポインタを整数アドレスへ変換し、ふたたびポインタへ戻す
///
/// アドレスの検査や記録に使うが、ポインタの生存期間には注意が必要。
pub fn pointer_roundtrip(value: &i32) -> i32 {
    let raw: *const i32 = value;
    let addr: usize = raw as usize;
    let recovered: *const i32 = addr as *const i32;
    // SAFETY: recoveredはvalueの有効なアドレス
    unsafe { *recovered }
}

/// ポインタの比較（大小比較も可能）
///
/// ポインタ比較は主にバッファ境界チェックや範囲確認に使われる。
/// 異なるオブジェクト間の大小比較は未定義動作になり得るが、
/// 同一バッファ内なら安全。
pub fn compare_pointers_in_slice(slice: &[i32]) -> bool {
    if slice.len() < 2 {
        return false;
    }
    let first: *const i32 = &slice[0];
    let last: *const i32 = &slice[slice.len() - 1];
    // 同じスライス内なのでアドレス比較は定義済み動作
    first < last
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_read_via_raw_pointer() {
        let x = 42_i32;
        assert_eq!(read_via_raw_pointer(&x), 42);
    }

    #[test]
    fn test_write_via_raw_pointer() {
        let mut x = 10_i32;
        write_via_raw_pointer(&mut x, 99);
        assert_eq!(x, 99);
    }

    #[test]
    fn test_null_pointer() {
        assert!(demonstrate_null_pointer());

        let non_null: *const i32 = &42;
        assert!(!non_null.is_null());
    }

    #[test]
    fn test_safe_deref_non_null() {
        let value = 7_i32;
        let ptr: *const i32 = &value;
        assert_eq!(safe_deref(ptr), Some(7));
    }

    #[test]
    fn test_safe_deref_null() {
        let null_ptr: *const i32 = ptr::null();
        assert_eq!(safe_deref(null_ptr), None);
    }

    #[test]
    fn test_pointers_equal() {
        let x = 1_i32;
        let p1: *const i32 = &x;
        let p2: *const i32 = &x;
        assert!(pointers_are_equal(p1, p2));

        let y = 1_i32;
        let p3: *const i32 = &y;
        assert!(!pointers_are_equal(p1, p3));
    }

    #[test]
    fn test_non_null() {
        let mut val = 55_i32;
        assert_eq!(demonstrate_non_null(&mut val), 55);
    }

    #[test]
    fn test_pointer_roundtrip() {
        let x = 123_i32;
        assert_eq!(pointer_roundtrip(&x), 123);
    }

    #[test]
    fn test_compare_pointers_in_slice() {
        let data = [1, 2, 3];
        assert!(compare_pointers_in_slice(&data));
        assert!(!compare_pointers_in_slice(&data[..1]));
    }

    #[test]
    fn test_const_and_mut_raw_pointer() {
        let mut data = [10_i32, 20, 30];
        let p: *mut i32 = data.as_mut_ptr();
        unsafe {
            // インデックスアクセスと同等
            assert_eq!(*p, 10);
            // 直接書き込み
            *p = 100;
        }
        assert_eq!(data[0], 100);
    }
}
