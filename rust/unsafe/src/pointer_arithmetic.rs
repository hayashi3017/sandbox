//! # ポインタ演算
//!
//! 生ポインタのアドレス計算に使う操作群です。
//! スライスやカスタムデータ構造の実装で頻繁に登場します。
//!
//! ## 主なメソッド
//!
//! | メソッド | 説明 | 安全条件 |
//! |---|---|---|
//! | `ptr.add(n)` | n要素分進む | resultが同じアロケーション内 |
//! | `ptr.sub(n)` | n要素分戻る | resultが同じアロケーション内 |
//! | `ptr.offset(n: isize)` | n要素分移動（正負可） | resultが同じアロケーション内 |
//! | `ptr.wrapping_add(n)` | オーバーフロー時はwrap（UBなし） | なし（でもderefは危険） |
//! | `ptr.byte_add(n)` | nバイト分進む | resultが同じアロケーション内 |
//! | `ptr.byte_sub(n)` | nバイト分戻る | resultが同じアロケーション内 |
//! | `ptr.offset_from(base)` | 2ポインタ間の要素数差 | 同じアロケーション内 |
//! | `ptr.byte_offset_from(base)` | 2ポインタ間のバイト差 | 同じアロケーション内 |


// ─── add / sub / offset の基本 ──────────────────────────────────────────────

/// スライスの要素をポインタ演算で順に合計する
pub fn sum_with_pointer_arithmetic(data: &[i32]) -> i32 {
    if data.is_empty() {
        return 0;
    }
    let mut sum = 0;
    let mut ptr = data.as_ptr();
    // SAFETY: endはスライスの「1つ後ろ」（デrefしないので有効）
    let end = unsafe { ptr.add(data.len()) };

    while ptr != end {
        // SAFETY: ptrは[data.as_ptr(), end)内
        sum += unsafe { *ptr };
        // SAFETY: 上と同じ
        ptr = unsafe { ptr.add(1) };
    }
    sum
}

/// `offset`（isizeバージョン）でインデックスアクセス
pub fn get_element_by_offset(data: &[i32], index: isize) -> Option<i32> {
    if index < 0 || index >= data.len() as isize {
        return None;
    }
    let ptr = data.as_ptr();
    // SAFETY: indexは[0, len)内なので有効
    Some(unsafe { *ptr.offset(index) })
}

// ─── wrapping_add（オーバーフロー対策） ─────────────────────────────────────

/// wrapping_addはオーバーフロー時にポインタ幅でwrapする
///
/// 通常のaddと異なり、未定義動作にならない。
/// ただし結果のポインタをderefすることは依然として危険。
pub fn pointer_wrapping_demo(ptr: *const u8) -> *const u8 {
    // これは決してderefしない（アドレス計算のデモ）
    ptr.wrapping_add(usize::MAX)
        .wrapping_add(1) // usize::MAX + 1 はwrapしてptrに戻る
}

// ─── byte_add / byte_sub ────────────────────────────────────────────────────

/// byte_addで型に関係なくバイト単位で移動する
///
/// 異なる型が混在する構造体（#[repr(C)]）のフィールドアクセスに使う。
#[repr(C)]
pub struct PackedHeader {
    pub magic: u32,
    pub version: u16,
    pub flags: u8,
}

pub fn read_version_from_header(header: &PackedHeader) -> u16 {
    let ptr = header as *const PackedHeader as *const u8;
    // magicフィールドは4バイトなので、versionはオフセット4バイト
    // SAFETY: headerは有効、u16のアライメントも満たす（#[repr(C)]のため）
    unsafe {
        let version_ptr = ptr.add(4) as *const u16;
        *version_ptr
    }
}

// ─── offset_from ────────────────────────────────────────────────────────────

/// 2つのポインタ間の要素数を計算
///
/// `ptr.offset_from(base)` = (ptr - base) / size_of::<T>()
pub fn elements_between<T>(start: *const T, end: *const T) -> isize {
    // SAFETY: startとendは同じアロケーション内（呼び出し元保証）
    unsafe { end.offset_from(start) }
}

/// スライス内の要素のインデックスを取得
pub fn index_of_element<T>(slice: &[T], element: &T) -> Option<usize> {
    let base = slice.as_ptr();
    let elem_ptr = element as *const T;

    // 範囲チェック
    let end = unsafe { base.add(slice.len()) };
    if elem_ptr < base || elem_ptr >= end {
        return None;
    }

    // SAFETY: elem_ptrはスライス内なので同じアロケーション
    let offset = unsafe { elem_ptr.offset_from(base) };
    Some(offset as usize)
}

// ─── ポインタの比較 ──────────────────────────────────────────────────────────

/// 2つのポインタが同じアロケーション内にあるかチェック（境界確認）
pub fn is_within_slice<T>(slice: &[T], ptr: *const T) -> bool {
    let start = slice.as_ptr();
    let end = unsafe { start.add(slice.len()) };
    ptr >= start && ptr < end
}

/// ポインタのソート（アドレス順）
pub fn sort_pointers<T>(ptrs: &mut [*const T]) {
    ptrs.sort_by(|a, b| (*a as usize).cmp(&(*b as usize)));
}

// ─── null許容ポインタのインクリメント ───────────────────────────────────────

/// nullでない場合のみポインタを進める
pub fn safe_advance<T>(ptr: *const T) -> *const T {
    if ptr.is_null() {
        ptr
    } else {
        // SAFETY: 呼び出し元がptrと次の要素が同じアロケーション内であることを保証
        unsafe { ptr.add(1) }
    }
}

// ─── ポインタを使ったカスタムイテレータ ─────────────────────────────────────

/// 生ポインタで実装したシンプルなイテレータ
pub struct RawSliceIter<T> {
    ptr: *const T,
    end: *const T,
}

impl<T> RawSliceIter<T> {
    pub fn new(slice: &[T]) -> Self {
        let ptr = slice.as_ptr();
        // SAFETY: slice.len()は有効な範囲内
        let end = unsafe { ptr.add(slice.len()) };
        RawSliceIter { ptr, end }
    }
}

impl<T: Copy> Iterator for RawSliceIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.ptr == self.end {
            return None;
        }
        // SAFETY: ptrはendより前にあるので有効
        let val = unsafe { *self.ptr };
        self.ptr = unsafe { self.ptr.add(1) };
        Some(val)
    }
}

// ─── ポインタ演算で二分探索 ──────────────────────────────────────────────────

/// 生ポインタを使った二分探索（標準ライブラリの実装に近い形）
pub fn binary_search_raw(data: &[i32], target: i32) -> Option<usize> {
    let mut lo = data.as_ptr();
    let mut hi = unsafe { lo.add(data.len()) };

    while lo < hi {
        let mid_offset = unsafe { hi.offset_from(lo) } as usize / 2;
        // SAFETY: mid_offsetはloとhiの間
        let mid = unsafe { lo.add(mid_offset) };
        // SAFETY: midは[data.as_ptr(), hi)内
        let val = unsafe { *mid };

        if val == target {
            // SAFETY: 同じアロケーション内
            let idx = unsafe { mid.offset_from(data.as_ptr()) } as usize;
            return Some(idx);
        } else if val < target {
            lo = unsafe { mid.add(1) };
        } else {
            hi = mid;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_with_ptr_arithmetic() {
        assert_eq!(sum_with_pointer_arithmetic(&[1, 2, 3, 4, 5]), 15);
        assert_eq!(sum_with_pointer_arithmetic(&[]), 0);
        assert_eq!(sum_with_pointer_arithmetic(&[-1, 1]), 0);
    }

    #[test]
    fn test_get_element_by_offset() {
        let data = [10, 20, 30, 40, 50];
        assert_eq!(get_element_by_offset(&data, 0), Some(10));
        assert_eq!(get_element_by_offset(&data, 4), Some(50));
        assert_eq!(get_element_by_offset(&data, 5), None);
        assert_eq!(get_element_by_offset(&data, -1), None);
    }

    #[test]
    fn test_wrapping_demo() {
        let base = 100_u8 as *const u8;
        let result = pointer_wrapping_demo(base);
        // wrapping_add(MAX).wrapping_add(1)はbaseに戻る
        assert_eq!(result, base);
    }

    #[test]
    fn test_read_version_from_header() {
        let header = PackedHeader {
            magic: 0xDEADBEEF,
            version: 42,
            flags: 0xFF,
        };
        assert_eq!(read_version_from_header(&header), 42);
    }

    #[test]
    fn test_elements_between() {
        let data = [1, 2, 3, 4, 5];
        let start = data.as_ptr();
        let end = unsafe { start.add(5) };
        assert_eq!(elements_between(start, end), 5);
        assert_eq!(elements_between(end, start), -5);
    }

    #[test]
    fn test_index_of_element() {
        let data = [10, 20, 30, 40, 50];
        assert_eq!(index_of_element(&data, &data[0]), Some(0));
        assert_eq!(index_of_element(&data, &data[4]), Some(4));

        let other = 10_i32;
        assert_eq!(index_of_element(&data, &other), None);
    }

    #[test]
    fn test_is_within_slice() {
        let data = [1, 2, 3, 4, 5];
        assert!(is_within_slice(&data, &data[2]));
        let other = 99_i32;
        assert!(!is_within_slice(&data, &other));
    }

    #[test]
    fn test_sort_pointers() {
        let data = [3_i32, 1, 4, 1, 5];
        let mut ptrs: Vec<*const i32> = data.iter().map(|x| x as *const i32).collect();
        sort_pointers(&mut ptrs);
        // ソート後はアドレス昇順（= 配列内順序）
        for i in 1..ptrs.len() {
            assert!(ptrs[i - 1] <= ptrs[i]);
        }
    }

    #[test]
    fn test_raw_slice_iter() {
        let data = [1_i32, 2, 3, 4, 5];
        let iter = RawSliceIter::new(&data);
        let collected: Vec<i32> = iter.collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_raw_slice_iter_empty() {
        let data: [i32; 0] = [];
        let iter = RawSliceIter::new(&data);
        let collected: Vec<i32> = iter.collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_binary_search_raw() {
        let data = [1, 3, 5, 7, 9, 11];
        assert_eq!(binary_search_raw(&data, 1), Some(0));
        assert_eq!(binary_search_raw(&data, 7), Some(3));
        assert_eq!(binary_search_raw(&data, 11), Some(5));
        assert_eq!(binary_search_raw(&data, 2), None);
        assert_eq!(binary_search_raw(&data, 0), None);
        assert_eq!(binary_search_raw(&data, 12), None);
    }
}
