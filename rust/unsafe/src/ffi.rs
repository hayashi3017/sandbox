//! # 外部関数インターフェース（FFI: Foreign Function Interface）
//!
//! FFIはRustから他言語（主にC）の関数を呼び出す、または
//! RustのコードをC言語から呼び出せるようにするための機能です。
//!
//! ## 主な要素
//!
//! - `extern "C" { fn ...; }` — C関数の宣言
//! - `#[no_mangle]` — Rustからエクスポートする関数名の修飾を無効化
//! - `extern "C" fn name()` — C ABIに従う関数の定義
//! - `CStr` / `CString` — Cの文字列との相互変換
//! - `libc` クレート — Cの型と標準ライブラリ関数
//!
//! ## C ABIの呼び出し規約
//!
//! `"C"` はシステムのデフォルトC ABIを使う。
//! Windowsでは `"stdcall"` が必要なケースもある。

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// ─── C標準ライブラリ関数の宣言 ─────────────────────────────────────────────

extern "C" {
    /// C標準ライブラリの `abs()`
    fn abs(x: c_int) -> c_int;

    /// C標準ライブラリの `strlen()`
    fn strlen(s: *const c_char) -> usize;

    /// C標準ライブラリの `atoi()`
    fn atoi(s: *const c_char) -> c_int;
}

// ─── Rustからのラッパー関数 ────────────────────────────────────────────────

/// Cの `abs()` を呼び出すsafeラッパー
pub fn c_abs(x: i32) -> i32 {
    // SAFETY: abs()はメモリを触らず、整数演算のみ
    unsafe { abs(x as c_int) as i32 }
}

/// Cの `strlen()` を呼び出すsafeラッパー
///
/// # Safety
///
/// `s`はnull終端されたUTF-8文字列へのポインタでなければならない。
/// 安全なAPIとして`CStr`経由で呼び出す。
pub fn c_strlen(s: &CStr) -> usize {
    // SAFETY: CStrはnull終端を保証
    unsafe { strlen(s.as_ptr()) }
}

/// Cの `atoi()` を呼び出すsafeラッパー
pub fn c_atoi(s: &CStr) -> i32 {
    // SAFETY: CStrはnull終端を保証、atoiはread-only
    unsafe { atoi(s.as_ptr()) as i32 }
}

// ─── CString / CStr の変換 ──────────────────────────────────────────────────

/// RustのStringからCStringを生成して長さを取得
pub fn string_length_via_c(s: &str) -> usize {
    // CString::newはnullバイトが含まれるとエラー
    let cstring = CString::new(s).expect("文字列にnullバイトが含まれています");
    c_strlen(&cstring)
}

/// CのバイトポインタからRustの&strに変換
///
/// # Safety
///
/// `ptr`はnull終端されたUTF-8文字列を指し、
/// 返り値の生存期間中有効でなければならない。
pub unsafe fn c_str_to_str<'a>(ptr: *const c_char) -> &'a str {
    // SAFETY: 呼び出し元がptrの有効性を保証
    let cstr = CStr::from_ptr(ptr);
    // SAFETY: UTF-8であることを前提
    std::str::from_utf8_unchecked(cstr.to_bytes())
}

// ─── Cからエクスポートする関数 ─────────────────────────────────────────────

/// Cから呼び出せるRustの関数
///
/// `#[no_mangle]`でシンボル名をそのまま保持し、
/// `extern "C"`でC ABIに合わせる。
#[no_mangle]
pub extern "C" fn rust_add(a: c_int, b: c_int) -> c_int {
    a + b
}

/// Cから呼び出せるRustの関数（文字列を受け取る）
///
/// # Safety
///
/// `s`はnull終端されたUTF-8文字列でなければならない。
#[no_mangle]
pub unsafe extern "C" fn rust_strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    CStr::from_ptr(s).to_bytes().len()
}

// ─── コールバック（関数ポインタ）のFFI ─────────────────────────────────────

/// C ABIのコールバック関数型
pub type CCallback = extern "C" fn(c_int) -> c_int;

/// コールバックを受け取るC関数のシミュレーション
///
/// 実際にはexternブロックで宣言するが、
/// ここではRust内でシミュレートする。
pub fn apply_c_callback(f: CCallback, x: i32) -> i32 {
    f(x as c_int) as i32
}

/// Rustで定義したC ABIコールバック
extern "C" fn double_it(x: c_int) -> c_int {
    x * 2
}

/// コールバックを渡して呼び出す例
pub fn use_callback(x: i32) -> i32 {
    apply_c_callback(double_it, x)
}

// ─── #[repr(C)] 構造体のFFI ─────────────────────────────────────────────────

/// Cと同じメモリレイアウトを持つ構造体
///
/// `#[repr(C)]`により、フィールドの順序・アライメントがCと一致する。
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Cから渡された`Point`を受け取るエクスポート関数
#[no_mangle]
pub extern "C" fn point_distance(p: Point) -> f64 {
    p.distance_from_origin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_abs() {
        assert_eq!(c_abs(-42), 42);
        assert_eq!(c_abs(10), 10);
        assert_eq!(c_abs(0), 0);
    }

    #[test]
    fn test_c_strlen() {
        let s = CString::new("hello").unwrap();
        assert_eq!(c_strlen(&s), 5);

        let empty = CString::new("").unwrap();
        assert_eq!(c_strlen(&empty), 0);
    }

    #[test]
    fn test_c_atoi() {
        let s = CString::new("123").unwrap();
        assert_eq!(c_atoi(&s), 123);

        let neg = CString::new("-456").unwrap();
        assert_eq!(c_atoi(&neg), -456);
    }

    #[test]
    fn test_string_length_via_c() {
        assert_eq!(string_length_via_c("hello"), 5);
        assert_eq!(string_length_via_c(""), 0);
        assert_eq!(string_length_via_c("Rust"), 4);
    }

    #[test]
    fn test_c_str_to_str() {
        let cstring = CString::new("test string").unwrap();
        let s = unsafe { c_str_to_str(cstring.as_ptr()) };
        assert_eq!(s, "test string");
    }

    #[test]
    fn test_rust_add() {
        assert_eq!(rust_add(3, 4), 7);
        assert_eq!(rust_add(-1, 1), 0);
    }

    #[test]
    fn test_use_callback() {
        assert_eq!(use_callback(5), 10);
        assert_eq!(use_callback(0), 0);
    }

    #[test]
    fn test_point() {
        let p = Point::new(3.0, 4.0);
        assert!((p.distance_from_origin() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_distance_export() {
        let p = Point::new(0.0, 5.0);
        assert!((point_distance(p) - 5.0).abs() < 1e-10);
    }
}
