//! # `mem::transmute` — メモリの再解釈
//!
//! `transmute<T, U>(val: T) -> U`はビットパターンを変えずに型を変える操作です。
//! コンパイル時に`size_of::<T>() == size_of::<U>()`を検証します。
//!
//! ## 主な用途
//!
//! | 操作 | transmute | 代替手段 |
//! |---|---|---|
//! | f32 ↔ u32 ビット変換 | `transmute(f)` | `f.to_bits()` / `f32::from_bits(u)` |
//! | &T → &U 型変換 | `transmute(r)` | `ptr::cast()` |
//! | 関数ポインタ変換 | `transmute(fp)` | なし（慎重に） |
//! | 生存期間の変更 | `transmute(r)` | 極めて危険 |
//!
//! ## 危険性
//!
//! - 型のアライメントが不一致 → 未定義動作
//! - 無効なビットパターン → 未定義動作（例: `bool`の0/1以外）
//! - 生存期間の延長 → use-after-free
//! - Dropが2回呼ばれる → double-free

use std::mem;

// ─── 基本的なビット変換 ─────────────────────────────────────────────────────

/// f32のビットをu32として読む
///
/// `f.to_bits()`が安全な代替手段。transmutableによるデモ。
pub fn f32_to_bits(f: f32) -> u32 {
    // SAFETY: f32とu32は同じ4バイト、すべてのビットパターンがu32として有効
    unsafe { mem::transmute::<f32, u32>(f) }
}

/// u32のビットをf32として解釈する
///
/// `f32::from_bits(u)`が安全な代替手段。
pub fn bits_to_f32(u: u32) -> f32 {
    // SAFETY: f32はすべてのビットパターンを受け入れる（NaNを含む）
    unsafe { mem::transmute::<u32, f32>(u) }
}

/// f64のビットをu64として読む
pub fn f64_to_bits(f: f64) -> u64 {
    // SAFETY: f64とu64は同じ8バイト
    unsafe { mem::transmute::<f64, u64>(f) }
}

// ─── 符号付き/符号なし整数の変換 ────────────────────────────────────────────

/// i32をu32に変換（`as`キャストと同等だが、transmute版）
pub fn i32_to_u32(i: i32) -> u32 {
    // SAFETY: i32とu32は同じ4バイト、2の補数表現が一致
    unsafe { mem::transmute::<i32, u32>(i) }
}

// ─── スライスへのポインタとfat pointer ──────────────────────────────────────

/// &[T]のfat pointer（データポインタ + 長さ）をタプルとして取り出す
pub fn slice_fat_pointer<T>(s: &[T]) -> (*const T, usize) {
    // SAFETY: &[T]はfat pointerで、(*const T, usize)と同じメモリレイアウト
    unsafe { mem::transmute::<&[T], (*const T, usize)>(s) }
}

/// タプルから&[T]を復元する
///
/// # Safety
///
/// `ptr`は`len`個のT要素が有効な領域を指し、
/// 返り値の生存期間中有効でなければならない。
pub unsafe fn fat_pointer_to_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    // SAFETY: 呼び出し元がptr, lenの有効性を保証
    mem::transmute::<(*const T, usize), &'a [T]>((ptr, len))
}

// ─── transmute_copy: Copyしてから変換 ───────────────────────────────────────

/// `transmute_copy`は参照からコピーして型変換する（サイズチェックなし）
///
/// `transmute`よりも危険。コピー元が変換先より小さい場合は未定義動作。
pub fn transmute_copy_demo(x: &u32) -> [u8; 4] {
    // SAFETY: u32は4バイト、[u8;4]も4バイト
    unsafe { mem::transmute_copy::<u32, [u8; 4]>(x) }
}

// ─── 関数ポインタの変換 ──────────────────────────────────────────────────────

/// fn(i32) -> i32 を fn(u32) -> u32 に変換
///
/// 呼び出し規約・サイズが一致している場合のみ安全。
pub fn transmute_fn_ptr(f: fn(i32) -> i32) -> fn(u32) -> u32 {
    // SAFETY: fn()ポインタはすべて同じサイズ（ポインタ幅）
    unsafe { mem::transmute::<fn(i32) -> i32, fn(u32) -> u32>(f) }
}

// ─── 生存期間の変更（extremely dangerous） ──────────────────────────────────

/// 生存期間を`'static`に拡張する（极めて危険な操作）
///
/// # Safety
///
/// `r`が参照する値は`'static`の生存期間を持たなければならない。
/// これを誤るとuse-after-freeが発生する。
/// 通常はこの関数を使う必要はない。
pub unsafe fn extend_lifetime<T>(r: &T) -> &'static T {
    mem::transmute::<&T, &'static T>(r)
}

// ─── 安全な代替手段との比較 ─────────────────────────────────────────────────

/// f32のビット変換: transmute vs to_bits()
pub fn compare_approaches_f32(f: f32) -> (u32, u32) {
    let via_transmute = f32_to_bits(f);
    let via_to_bits = f.to_bits();
    (via_transmute, via_to_bits)
}

/// 配列とスライスの変換（transmute不使用の安全版）
pub fn array_to_slice_safe(arr: &[u8; 4]) -> &[u8] {
    arr.as_slice()
}

/// [i32; 4]を[u32; 4]にtransmute（ビットパターン保持）
///
/// &[u8;4]→&[u8]はポインタのサイズが異なるためtransmute不可。
/// 配列の中身同士は同じサイズなので変換できる。
pub fn i32_array_to_u32_array(arr: [i32; 4]) -> [u32; 4] {
    // SAFETY: i32とu32は同じ4バイト、2の補数表現が一致
    unsafe { mem::transmute::<[i32; 4], [u32; 4]>(arr) }
}

// ─── ゼロ初期化 ─────────────────────────────────────────────────────────────

/// 型をゼロで初期化する
///
/// `Default::default()`や`MaybeUninit::zeroed()`の方が安全。
/// ゼロが有効でない型（例: 参照、bool以外の値）には使えない。
pub fn zeroed_u64() -> u64 {
    // SAFETY: u64はすべてのビットパターンが有効
    unsafe { mem::zeroed::<u64>() }
}

/// `MaybeUninit`を使った安全なゼロ初期化
pub fn zeroed_u64_safe() -> u64 {
    let mu = mem::MaybeUninit::<u64>::zeroed();
    // SAFETY: u64のゼロ表現は有効
    unsafe { mu.assume_init() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_bits_roundtrip() {
        let f = 3.14_f32;
        let bits = f32_to_bits(f);
        let recovered = bits_to_f32(bits);
        assert_eq!(f, recovered);
    }

    #[test]
    fn test_f32_bits_matches_to_bits() {
        let f = 1.0_f32;
        let (via_transmute, via_to_bits) = compare_approaches_f32(f);
        assert_eq!(via_transmute, via_to_bits);
        assert_eq!(via_transmute, 0x3F800000);
    }

    #[test]
    fn test_f32_special_values() {
        // NaN
        let nan_bits = f32_to_bits(f32::NAN);
        let recovered_nan = bits_to_f32(nan_bits);
        assert!(recovered_nan.is_nan());

        // Infinity
        let inf_bits = f32_to_bits(f32::INFINITY);
        assert_eq!(inf_bits, 0x7F800000);

        // -0.0
        let neg_zero_bits = f32_to_bits(-0.0_f32);
        assert_eq!(neg_zero_bits, 0x80000000);
    }

    #[test]
    fn test_f64_bits() {
        let f = 1.0_f64;
        let bits = f64_to_bits(f);
        assert_eq!(bits, 0x3FF0000000000000);
    }

    #[test]
    fn test_i32_to_u32() {
        assert_eq!(i32_to_u32(-1), u32::MAX);
        assert_eq!(i32_to_u32(0), 0);
        assert_eq!(i32_to_u32(i32::MAX), i32::MAX as u32);
    }

    #[test]
    fn test_slice_fat_pointer() {
        let data = [1_i32, 2, 3, 4, 5];
        let slice: &[i32] = &data;
        let (ptr, len) = slice_fat_pointer(slice);
        assert_eq!(len, 5);
        // ptrはスライスの先頭と一致
        assert_eq!(ptr, slice.as_ptr());
    }

    #[test]
    fn test_fat_pointer_to_slice() {
        let data = [10_i32, 20, 30];
        let ptr = data.as_ptr();
        let slice = unsafe { fat_pointer_to_slice(ptr, 3) };
        assert_eq!(slice, &[10, 20, 30]);
    }

    #[test]
    fn test_transmute_copy() {
        let x: u32 = 0x01020304;
        let bytes = transmute_copy_demo(&x);
        // リトルエンディアンの場合
        #[cfg(target_endian = "little")]
        assert_eq!(bytes, [0x04, 0x03, 0x02, 0x01]);
        #[cfg(target_endian = "big")]
        assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_transmute_fn_ptr() {
        fn negate(x: i32) -> i32 {
            -x
        }
        let f: fn(u32) -> u32 = transmute_fn_ptr(negate);
        // -1_i32 as u32 = u32::MAX, -(u32::MAX as i32) is u32::MAX when reinterpreted
        let result = f(1_u32); // negate(1_i32) = -1_i32 = u32::MAX when transmuted
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn test_i32_array_to_u32_array() {
        let arr = [-1_i32, 0, 1, i32::MAX];
        let u = i32_array_to_u32_array(arr);
        assert_eq!(u[0], u32::MAX);
        assert_eq!(u[1], 0);
        assert_eq!(u[2], 1);
        assert_eq!(u[3], i32::MAX as u32);
    }

    #[test]
    fn test_zeroed() {
        assert_eq!(zeroed_u64(), 0);
        assert_eq!(zeroed_u64_safe(), 0);
    }
}
