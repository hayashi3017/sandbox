//! # 手動メモリ管理
//!
//! Rustは通常、所有権システムによってメモリを自動管理しますが、
//! `unsafe`を使って手動でメモリを制御することもできます。
//!
//! ## 主なAPI
//!
//! | API | 説明 |
//! |---|---|
//! | `Box::into_raw(b)` | BoxからRaw pointerを取り出す（所有権を放棄） |
//! | `Box::from_raw(p)` | Raw pointerからBoxを再構築（所有権を取得） |
//! | `mem::forget(v)` | dropを呼ばずに値を破棄（リーク） |
//! | `ManuallyDrop<T>` | dropのタイミングを手動制御 |
//! | `ptr::read(p)` | ポインタから値をコピー（移動） |
//! | `ptr::write(p, v)` | ポインタに値を書き込む（dropなし） |
//! | `ptr::drop_in_place(p)` | ポインタが指す値のdropを呼ぶ |
//! | `ptr::copy(s, d, n)` | n要素コピー（重複可） |
//! | `ptr::copy_nonoverlapping` | n要素コピー（重複不可、高速） |
//! | `alloc::alloc(layout)` | グローバルアロケータで確保 |
//! | `alloc::dealloc(p, layout)` | グローバルアロケータで解放 |

use std::alloc::{self, Layout};
use std::mem::{self, ManuallyDrop};
use std::ptr;

// ─── Box::into_raw / Box::from_raw ──────────────────────────────────────────

/// BoxをRaw pointerに変換し、後で回収する
///
/// `into_raw`した後は`from_raw`で必ず回収しないとメモリリークになる。
pub fn box_raw_roundtrip(val: i32) -> i32 {
    let boxed = Box::new(val);
    let raw: *mut i32 = Box::into_raw(boxed);

    // rawポインタを通じて操作
    // SAFETY: rawはBoxから取ったので有効
    unsafe {
        *raw += 1;
        let recovered = Box::from_raw(raw); // 再びBoxとして管理
        *recovered
    }
}

/// Raw pointerを返す関数（呼び出し元がfrom_rawで解放する責任）
pub fn allocate_value(val: String) -> *mut String {
    Box::into_raw(Box::new(val))
}

/// Raw pointerを受け取り、Boxとして解放する
///
/// # Safety
///
/// `ptr`は`allocate_value`で返したポインタでなければならない。
pub unsafe fn free_value(ptr: *mut String) {
    drop(Box::from_raw(ptr));
}

// ─── mem::forget ────────────────────────────────────────────────────────────

/// `forget`を使ってdropをスキップする
///
/// FFI等でCが所有権を持つ場合に使う。使いすぎるとメモリリーク。
pub fn demonstrate_forget() -> usize {
    let v = vec![1_i32, 2, 3, 4, 5];
    let len = v.len();
    let ptr = v.as_ptr();

    // vをforgetしてdropをスキップ
    mem::forget(v);

    // ptrはまだ（一時的に）有効だが、もはや誰も所有していない
    // SAFETY: forgetの直後でまだ有効（ただしこれは教育目的のデモ）
    let _check = unsafe { *ptr };
    len // 実際にはptrを使い続けてはいけない
}

// ─── ManuallyDrop ───────────────────────────────────────────────────────────

/// `ManuallyDrop`でdropを制御する
///
/// `ManuallyDrop::into_inner()`を呼ぶと値が取り出されdropされる。
/// 呼ばなければdropはスキップされる。
pub fn manual_drop_demo() -> String {
    let s = ManuallyDrop::new(String::from("hello"));
    // このままスコープを出てもdropされない（リーク）

    // 明示的に取り出してdrop
    ManuallyDrop::into_inner(s)
}

/// ManuallyDropを使った条件付きdrop
pub fn conditional_drop(flag: bool) -> usize {
    let v = ManuallyDrop::new(vec![1, 2, 3]);
    let len = v.len();

    if flag {
        // SAFETY: ManuallyDropにより一度だけdropを呼ぶ
        unsafe { ManuallyDrop::drop(&mut ManuallyDrop::new(ManuallyDrop::into_inner(v))) };
    }
    // flagがfalseならdropされない（リーク）

    len
}

// ─── ptr::read / ptr::write ──────────────────────────────────────────────────

/// `ptr::read`でポインタから値をコピーする
///
/// 元のポインタが指すメモリはまだ初期化されているが、
/// 所有権を奪っているため、二重dropに注意。
pub fn read_value_from_ptr() -> String {
    let mut s = ManuallyDrop::new(String::from("world"));
    let ptr: *mut String = &mut *s;

    // SAFETY: ptrは有効なStringを指す
    let owned: String = unsafe { ptr::read(ptr) };
    // sのManuallyDropがdropをスキップするので、ownedだけがdropされる
    owned
}

/// `ptr::write`でdropを呼ばずにポインタに値を書き込む
///
/// ポインタが既初期化の値を指している場合、元の値はdropされない（リーク）ので注意。
pub fn write_to_uninitialized() -> i32 {
    let mut uninit = mem::MaybeUninit::<i32>::uninit();
    // SAFETY: uninitポインタへの書き込み（既存値のdropは不要）
    unsafe {
        ptr::write(uninit.as_mut_ptr(), 42);
        uninit.assume_init()
    }
}

// ─── ptr::drop_in_place ──────────────────────────────────────────────────────

/// `ptr::drop_in_place`でポインタが指す値をdropする
///
/// 値はメモリから消えるわけではなく、デストラクタを実行するだけ。
/// drop後のメモリへのアクセスは未定義動作。
pub fn drop_in_place_demo(s: String) {
    let mut md = ManuallyDrop::new(s);
    // SAFETY: md.as_mut_ptr()は有効なStringを指す
    unsafe {
        ptr::drop_in_place(md.as_mut_ptr());
    }
    // mdはManuallyDropなので二重dropは発生しない
}

// ─── ptr::copy / ptr::copy_nonoverlapping ────────────────────────────────────

/// overlappingなメモリコピー（memmoveに相当）
pub fn overlapping_copy(data: &mut [i32], src: usize, dst: usize, count: usize) {
    // SAFETY: スライス内なので有効なメモリ、countが範囲内
    unsafe {
        ptr::copy(data.as_ptr().add(src), data.as_mut_ptr().add(dst), count);
    }
}

/// non-overlappingなメモリコピー（memcpyに相当、より高速）
pub fn nonoverlapping_copy(src: &[i32], dst: &mut [i32]) {
    assert_eq!(src.len(), dst.len());
    // SAFETY: src/dstは別々のスライスなのでnon-overlapping
    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

// ─── グローバルアロケータ ─────────────────────────────────────────────────────

/// グローバルアロケータで手動メモリ確保
///
/// Cの`malloc`/`free`に相当。
/// 通常はBoxやVecを使うべき。
pub fn manual_alloc_demo() -> i32 {
    let layout = Layout::new::<i32>();

    // SAFETY: layoutは有効
    let ptr = unsafe { alloc::alloc(layout) as *mut i32 };
    assert!(!ptr.is_null(), "アロケーション失敗");

    // SAFETY: ptrは有効なi32サイズのメモリを指す
    unsafe {
        ptr::write(ptr, 777);
        let val = ptr::read(ptr);
        alloc::dealloc(ptr as *mut u8, layout);
        val
    }
}

/// 配列サイズのメモリを手動確保
pub fn manual_alloc_array(count: usize) -> Vec<i32> {
    if count == 0 {
        return Vec::new();
    }

    let layout = Layout::array::<i32>(count).expect("レイアウト計算失敗");
    let ptr = unsafe { alloc::alloc_zeroed(layout) as *mut i32 };
    assert!(!ptr.is_null(), "アロケーション失敗");

    // ゼロ初期化済みなのでそのままスライスとして扱える
    // SAFETY: alloc_zeroedで初期化済み、layoutのサイズが正しい
    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr, count);
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as i32;
        }
        let result = slice.to_vec();
        alloc::dealloc(ptr as *mut u8, layout);
        result
    }
}

// ─── MaybeUninit ─────────────────────────────────────────────────────────────

/// `MaybeUninit`で未初期化メモリを安全に扱う
///
/// `mem::uninitialized()`は廃止済み。代わりに`MaybeUninit`を使う。
pub fn init_array_with_maybe_uninit(n: usize) -> Vec<u32> {
    let mut v: Vec<mem::MaybeUninit<u32>> = Vec::with_capacity(n);

    for i in 0..n {
        v.push(mem::MaybeUninit::new(i as u32 * 2));
    }

    // SAFETY: すべての要素をMaybeUninit::newで初期化した
    v.into_iter().map(|mu| unsafe { mu.assume_init() }).collect()
}

/// 配列をMaybeUninitで初期化（ゼロコピー）
pub fn init_fixed_array() -> [u32; 5] {
    let mut arr: [mem::MaybeUninit<u32>; 5] = unsafe {
        // SAFETY: MaybeUninit配列は「初期化不要」なので配列全体のuninitは安全
        mem::MaybeUninit::uninit().assume_init()
    };

    for (i, slot) in arr.iter_mut().enumerate() {
        slot.write(i as u32);
    }

    // SAFETY: すべてのスロットに書き込んだ
    unsafe { mem::transmute::<[mem::MaybeUninit<u32>; 5], [u32; 5]>(arr) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_raw_roundtrip() {
        assert_eq!(box_raw_roundtrip(10), 11);
    }

    #[test]
    fn test_allocate_free_value() {
        let ptr = allocate_value("test".to_string());
        unsafe {
            assert_eq!(*ptr, "test");
            free_value(ptr);
        }
    }

    #[test]
    fn test_forget() {
        let len = demonstrate_forget();
        assert_eq!(len, 5);
    }

    #[test]
    fn test_manual_drop() {
        let s = manual_drop_demo();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_conditional_drop() {
        let len = conditional_drop(true);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_read_value_from_ptr() {
        let s = read_value_from_ptr();
        assert_eq!(s, "world");
    }

    #[test]
    fn test_write_to_uninitialized() {
        assert_eq!(write_to_uninitialized(), 42);
    }

    #[test]
    fn test_drop_in_place_demo() {
        drop_in_place_demo("drop me".to_string());
        // パニックしなければOK
    }

    #[test]
    fn test_overlapping_copy() {
        let mut data = [1, 2, 3, 4, 5];
        // data[0..3]をdata[1..4]に重複コピー（後ろにシフト）
        overlapping_copy(&mut data, 0, 1, 3);
        assert_eq!(data[1], 1);
        assert_eq!(data[2], 2);
        assert_eq!(data[3], 3);
    }

    #[test]
    fn test_nonoverlapping_copy() {
        let src = [10, 20, 30];
        let mut dst = [0; 3];
        nonoverlapping_copy(&src, &mut dst);
        assert_eq!(dst, [10, 20, 30]);
    }

    #[test]
    fn test_manual_alloc() {
        assert_eq!(manual_alloc_demo(), 777);
    }

    #[test]
    fn test_manual_alloc_array() {
        let v = manual_alloc_array(5);
        assert_eq!(v, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_maybe_uninit_vec() {
        let v = init_array_with_maybe_uninit(4);
        assert_eq!(v, [0, 2, 4, 6]);
    }

    #[test]
    fn test_init_fixed_array() {
        let arr = init_fixed_array();
        assert_eq!(arr, [0, 1, 2, 3, 4]);
    }
}
