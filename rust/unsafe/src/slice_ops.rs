//! # 生ポインタからのスライス/文字列生成
//!
//! `std::slice::from_raw_parts`等を使って、生ポインタと長さから
//! スライスや文字列を生成する操作です。
//!
//! ## 主なAPI
//!
//! | 関数 | 説明 |
//! |---|---|
//! | `slice::from_raw_parts(ptr, len)` | `*const T` + 長さ → `&[T]` |
//! | `slice::from_raw_parts_mut(ptr, len)` | `*mut T` + 長さ → `&mut [T]` |
//! | `str::from_utf8_unchecked(bytes)` | `&[u8]` → `&str`（UTF-8検証なし） |
//! | `String::from_raw_parts(ptr, len, cap)` | 生ポインタから`String`を再構築 |
//!
//! ## 安全条件（from_raw_parts）
//!
//! 1. `ptr`は非null、適切にアライメントされている
//! 2. `ptr`は`len * size_of::<T>()`バイトの連続したメモリを指す
//! 3. そのメモリは有効に初期化されている
//! 4. 返り値の生存期間中、メモリは変更されない（不変スライスの場合）
//! 5. `len <= isize::MAX`

use std::slice;
use std::str;

// ─── from_raw_parts の基本 ──────────────────────────────────────────────────

/// 生ポインタと長さから不変スライスを生成
///
/// # Safety
///
/// `ptr`は`len`個の`T`が格納された有効なメモリを指さなければならない。
pub unsafe fn slice_from_ptr<T>(ptr: *const T, len: usize) -> &'static [T] {
    slice::from_raw_parts(ptr, len)
}

/// 生ポインタと長さから可変スライスを生成
///
/// # Safety
///
/// `ptr`は`len`個の`T`が格納された有効なメモリを指し、
/// 返り値の生存期間中、他に参照が存在してはならない。
pub unsafe fn slice_from_ptr_mut<T>(ptr: *mut T, len: usize) -> &'static mut [T] {
    slice::from_raw_parts_mut(ptr, len)
}

// ─── スライスの分割（split_at_unchecked） ──────────────────────────────────

/// スライスを境界チェックなしで分割
///
/// `slice.split_at(mid)`の高速版。midが範囲内であることは呼び出し元が保証。
pub fn split_unchecked<T>(s: &[T], mid: usize) -> (&[T], &[T]) {
    // SAFETY: mid <= s.len()を呼び出し元が保証
    unsafe {
        let left = slice::from_raw_parts(s.as_ptr(), mid);
        let right = slice::from_raw_parts(s.as_ptr().add(mid), s.len() - mid);
        (left, right)
    }
}

// ─── 文字列操作 ─────────────────────────────────────────────────────────────

/// バイトスライスをUTF-8検証なしで&strに変換
///
/// # Safety
///
/// `bytes`は有効なUTF-8でなければならない。
/// 無効なUTF-8を渡すと、後続の文字列操作で未定義動作が発生する可能性がある。
pub unsafe fn bytes_to_str_unchecked(bytes: &[u8]) -> &str {
    str::from_utf8_unchecked(bytes)
}

/// 安全版: UTF-8検証あり
pub fn bytes_to_str_safe(bytes: &[u8]) -> Result<&str, std::str::Utf8Error> {
    str::from_utf8(bytes)
}

// ─── カスタムバッファからのスライス生成 ─────────────────────────────────────

/// C言語から受け取ったバッファをRustのスライスとして扱う
///
/// FFIバウンダリでよく使うパターン。
///
/// # Safety
///
/// `buf`は`len`バイトの有効なメモリを指さなければならない。
pub unsafe fn c_buffer_to_slice(buf: *const u8, len: usize) -> &'static [u8] {
    slice::from_raw_parts(buf, len)
}

// ─── スライスの重複部分アクセス ─────────────────────────────────────────────

/// 同一スライスの重なりのない2つの可変部分を同時に借用
///
/// Rustの借用チェッカーでは、同じスライスから2つの可変参照を取れないが、
/// 重なりがないことが分かっている場合にunsafeで実現できる。
pub fn split_two_mut<T>(s: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j, "インデックスが同じです");
    assert!(i < s.len() && j < s.len(), "インデックスが範囲外です");

    let ptr = s.as_mut_ptr();
    // SAFETY: i != j なのでエイリアスは発生しない
    unsafe {
        let a = &mut *ptr.add(i);
        let b = &mut *ptr.add(j);
        (a, b)
    }
}

// ─── スライスのアライメント分割 ─────────────────────────────────────────────

/// u8スライスをu32のアライメント境界で分割する
///
/// `align_to`はstableなsafeAPIだが、内部でfrom_raw_partsを使っている。
/// 使い方を示す。
pub fn align_u8_to_u32(s: &[u8]) -> (&[u8], &[u32], &[u8]) {
    // SAFETY: u8→u32の変換、アライメントをalign_toが保証
    unsafe { s.align_to::<u32>() }
}

// ─── ボックス化スライス ─────────────────────────────────────────────────────

/// 生ポインタからBoxed sliceを復元
///
/// `Box::from_raw`と`slice::from_raw_parts_mut`を組み合わせる。
///
/// # Safety
///
/// `ptr`は`len`個のTを格納するBox::into_rawで得たポインタでなければならない。
pub unsafe fn box_slice_from_raw<T>(ptr: *mut T, len: usize) -> Box<[T]> {
    let slice = slice::from_raw_parts_mut(ptr, len);
    Box::from_raw(slice)
}

// ─── 固定長配列のスライス化 ─────────────────────────────────────────────────

/// 配列の生ポインタからスライスを生成（常にsafe）
pub fn array_as_slice<T, const N: usize>(arr: &[T; N]) -> &[T] {
    // これはsafe operationだがfrom_raw_partsの簡単な例として
    unsafe { slice::from_raw_parts(arr.as_ptr(), N) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_from_ptr() {
        let data = [1_i32, 2, 3, 4, 5];
        let s = unsafe { slice_from_ptr(data.as_ptr(), data.len()) };
        assert_eq!(s, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_slice_from_ptr_mut() {
        let mut data = [1_i32, 2, 3];
        let s = unsafe { slice_from_ptr_mut(data.as_mut_ptr(), data.len()) };
        s[0] = 100;
        assert_eq!(data[0], 100);
    }

    #[test]
    fn test_split_unchecked() {
        let data = [1, 2, 3, 4, 5];
        let (left, right) = split_unchecked(&data, 2);
        assert_eq!(left, &[1, 2]);
        assert_eq!(right, &[3, 4, 5]);
    }

    #[test]
    fn test_split_unchecked_empty() {
        let data: [i32; 0] = [];
        let (left, right) = split_unchecked(&data, 0);
        assert_eq!(left, &[] as &[i32]);
        assert_eq!(right, &[] as &[i32]);
    }

    #[test]
    fn test_bytes_to_str_unchecked() {
        let bytes = b"hello, world";
        let s = unsafe { bytes_to_str_unchecked(bytes) };
        assert_eq!(s, "hello, world");
    }

    #[test]
    fn test_bytes_to_str_safe() {
        let valid = b"Rust\xE3\x81\x99\xE3\x81\x93\xE3\x81\x84"; // UTF-8
        assert!(bytes_to_str_safe(valid).is_ok());

        let invalid = &[0xFF_u8, 0xFE];
        assert!(bytes_to_str_safe(invalid).is_err());
    }

    #[test]
    fn test_split_two_mut() {
        let mut data = [10, 20, 30, 40, 50];
        let (a, b) = split_two_mut(&mut data, 1, 3);
        *a = 200;
        *b = 400;
        assert_eq!(data, [10, 200, 30, 400, 50]);
    }

    #[test]
    #[should_panic]
    fn test_split_two_mut_same_index() {
        let mut data = [1, 2, 3];
        split_two_mut(&mut data, 1, 1);
    }

    #[test]
    fn test_align_u8_to_u32() {
        let data: Vec<u8> = vec![0_u8; 16];
        let (prefix, aligned, suffix) = align_u8_to_u32(&data);
        assert!(prefix.len() + aligned.len() * 4 + suffix.len() == 16);
    }

    #[test]
    fn test_array_as_slice() {
        let arr = [1_i32, 2, 3, 4];
        let s = array_as_slice(&arr);
        assert_eq!(s, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_box_slice_from_raw() {
        let original: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
        let ptr = Box::into_raw(original) as *mut i32;
        let recovered = unsafe { box_slice_from_raw(ptr, 3) };
        assert_eq!(&*recovered, &[1, 2, 3]);
        // recoveredはBoxなのでここでdropされる
    }
}
