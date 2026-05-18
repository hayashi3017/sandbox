//! # 可変静的変数（`static mut`）
//!
//! `static mut`はグローバルな可変状態を持つ変数です。
//! マルチスレッド環境では**データ競合**の危険があるため、
//! アクセスはunsafeブロックが必要です。
//!
//! ## 代替手段（より安全）
//!
//! | 方法 | 適用場面 |
//! |---|---|
//! | `std::sync::Mutex<T>` | 汎用の排他アクセス |
//! | `std::sync::RwLock<T>` | 読み取り多数・書き込み少数 |
//! | `std::sync::atomic::AtomicXxx` | 整数・ブール値のアトミック操作 |
//! | `once_cell::sync::OnceCell<T>` | 遅延初期化 |
//!
//! ## 注意
//!
//! Rust 2024 editionでは`static mut`への参照取得が**hard error**になります。
//! このモジュールはeducational目的のため2021 editionで記述しています。
//! 本番コードでは`static mut`の代わりに上記の代替手段を使ってください。

use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;

// ─── static mut の基本 ──────────────────────────────────────────────────────

static mut COUNTER: i32 = 0;

/// グローバルカウンターをインクリメントする（シングルスレッド想定）
///
/// # Safety
///
/// この関数はシングルスレッドでのみ安全。
/// マルチスレッド環境での呼び出しはデータ競合を引き起こす。
pub unsafe fn increment_counter() {
    COUNTER += 1;
}

/// グローバルカウンターを読む（シングルスレッド想定）
///
/// # Safety
///
/// シングルスレッドでのみ安全。
pub unsafe fn get_counter() -> i32 {
    COUNTER
}

/// グローバルカウンターをリセット
///
/// # Safety
///
/// シングルスレッドでのみ安全。
pub unsafe fn reset_counter() {
    COUNTER = 0;
}

// ─── static mut の配列 ──────────────────────────────────────────────────────

static mut BUFFER: [u8; 16] = [0u8; 16];

/// 固定サイズバッファへの書き込み（C FFIとのインターフェース等で使う）
///
/// # Safety
///
/// シングルスレッドでのみ安全。`offset < 16`でなければならない。
pub unsafe fn write_to_buffer(offset: usize, byte: u8) {
    assert!(offset < 16, "バッファオーバーフロー");
    BUFFER[offset] = byte;
}

/// # Safety
///
/// シングルスレッドでのみ安全。`offset < 16`でなければならない。
pub unsafe fn read_from_buffer(offset: usize) -> u8 {
    assert!(offset < 16, "バッファ境界外");
    BUFFER[offset]
}

// ─── static mut の代替: AtomicXxx ───────────────────────────────────────────

/// `AtomicI32`を使ったスレッドセーフなカウンター
///
/// unsafeを使わずマルチスレッドで安全に操作できる。
static ATOMIC_COUNTER: AtomicI32 = AtomicI32::new(0);

pub fn atomic_increment() {
    ATOMIC_COUNTER.fetch_add(1, Ordering::SeqCst);
}

pub fn atomic_get() -> i32 {
    ATOMIC_COUNTER.load(Ordering::SeqCst)
}

pub fn atomic_reset() {
    ATOMIC_COUNTER.store(0, Ordering::SeqCst);
}

/// Compare-And-Swap: 期待値と一致する場合のみ更新
pub fn atomic_compare_exchange(expected: i32, new: i32) -> Result<i32, i32> {
    ATOMIC_COUNTER.compare_exchange(expected, new, Ordering::SeqCst, Ordering::Relaxed)
}

// ─── AtomicUsize (ポインタサイズ) ───────────────────────────────────────────

static GLOBAL_ID: AtomicUsize = AtomicUsize::new(0);

/// スレッドセーフなID発番器
pub fn next_id() -> usize {
    GLOBAL_ID.fetch_add(1, Ordering::Relaxed)
}

// ─── static mut の代替: Mutex ───────────────────────────────────────────────

/// `Mutex<Vec<T>>`を使ったスレッドセーフなグローバルリスト
static GLOBAL_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn log_message(msg: &str) {
    GLOBAL_LOG.lock().unwrap().push(msg.to_string());
}

pub fn get_log() -> Vec<String> {
    GLOBAL_LOG.lock().unwrap().clone()
}

pub fn clear_log() {
    GLOBAL_LOG.lock().unwrap().clear();
}

// ─── 遅延初期化パターン ─────────────────────────────────────────────────────

/// `static mut`による手動遅延初期化（危険）
///
/// 本番では`std::sync::OnceLock`を使うべき。
static mut LAZY_VALUE: Option<i32> = None;

/// # Safety
///
/// シングルスレッドでのみ安全。初期化は一度だけ行われる。
pub unsafe fn get_or_init(init: i32) -> i32 {
    if LAZY_VALUE.is_none() {
        LAZY_VALUE = Some(init);
    }
    LAZY_VALUE.unwrap()
}

/// `std::sync::OnceLock`を使った安全な遅延初期化
use std::sync::OnceLock;

static SAFE_LAZY: OnceLock<String> = OnceLock::new();

pub fn get_or_init_safe(init: &str) -> &'static str {
    SAFE_LAZY.get_or_init(|| init.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // static mutのテストはシングルスレッドで実行する必要がある。
    // cargo test はデフォルトでスレッドを並列実行するため、
    // `-- --test-threads=1` オプションか `#[serial]` クレートが必要だが、
    // ここでは各テスト内でリセットすることで対処。

    #[test]
    fn test_counter_basic() {
        unsafe {
            reset_counter();
            increment_counter();
            increment_counter();
            assert_eq!(get_counter(), 2);
            reset_counter();
            assert_eq!(get_counter(), 0);
        }
    }

    #[test]
    fn test_buffer() {
        unsafe {
            write_to_buffer(0, 0xAB);
            write_to_buffer(1, 0xCD);
            assert_eq!(read_from_buffer(0), 0xAB);
            assert_eq!(read_from_buffer(1), 0xCD);
        }
    }

    #[test]
    fn test_atomic_counter() {
        atomic_reset();
        atomic_increment();
        atomic_increment();
        atomic_increment();
        assert_eq!(atomic_get(), 3);
        atomic_reset();
    }

    #[test]
    fn test_compare_exchange() {
        atomic_reset();
        let result = atomic_compare_exchange(0, 42);
        assert_eq!(result, Ok(0));
        assert_eq!(atomic_get(), 42);

        let fail = atomic_compare_exchange(0, 99);
        assert_eq!(fail, Err(42));
        atomic_reset();
    }

    #[test]
    fn test_global_log() {
        clear_log();
        log_message("hello");
        log_message("world");
        let log = get_log();
        assert!(log.contains(&"hello".to_string()));
        assert!(log.contains(&"world".to_string()));
        clear_log();
    }

    #[test]
    fn test_safe_lazy() {
        let val = get_or_init_safe("initialized");
        assert_eq!(val, "initialized");
        // 2回目は同じ値が返る
        let val2 = get_or_init_safe("other");
        assert_eq!(val2, "initialized");
    }

    #[test]
    #[should_panic]
    fn test_buffer_overflow() {
        unsafe {
            write_to_buffer(16, 0xFF); // 境界外アクセス -> panic
        }
    }
}
