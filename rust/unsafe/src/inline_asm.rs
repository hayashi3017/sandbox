//! # インラインアセンブリ（`asm!` / `global_asm!`）
//!
//! Rustはアセンブリを直接インライン記述する機能を提供します。
//! アーキテクチャ固有の命令やレジスタを使う場合に必要です。
//!
//! ## マクロの種類
//!
//! | マクロ | 説明 |
//! |---|---|
//! | `asm!` | 関数内のインラインアセンブリ（unsafe要） |
//! | `global_asm!` | モジュールレベルのアセンブリ（unsafe不要） |
//!
//! ## `asm!`の構文
//!
//! ```text
//! asm!(
//!     "命令テンプレート",
//!     入力/出力オペランド,
//!     オプション
//! );
//! ```
//!
//! ## オペランド制約
//!
//! | 制約 | 意味 |
//! |---|---|
//! | `in(reg) val` | 入力（汎用レジスタ） |
//! | `out(reg) val` | 出力 |
//! | `inout(reg) val` | 入出力 |
//! | `in("rax") val` | 特定レジスタ指定 |
//! | `lateout(reg) val` | 遅延出力（入力と同じレジスタを使える） |
//! | `const N` | コンパイル時定数 |
//! | `sym SYMBOL` | シンボル参照 |
//!
//! ## オプション
//!
//! | オプション | 意味 |
//! |---|---|
//! | `nostack` | スタックを使わない |
//! | `nomem` | メモリを読み書きしない |
//! | `pure` | 副作用なし（同じ入力には同じ出力） |
//! | `preserves_flags` | フラグレジスタを変更しない |
//! | `att_syntax` | AT&T構文（デフォルトはIntel構文） |

// ─── x86_64 固有の実装 ──────────────────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
pub mod x86_64 {
    use std::arch::asm;

    /// アセンブリでNOP（何もしない）命令を実行
    pub fn nop() {
        unsafe {
            asm!("nop", options(nostack, nomem, preserves_flags));
        }
    }

    /// asmでの加算（デモ目的）
    ///
    /// x86_64の`add`命令を使って2つの数を足す。
    pub fn add_asm(a: u64, b: u64) -> u64 {
        let result: u64;
        unsafe {
            asm!(
                "add {0}, {1}",
                inout(reg) a => result,
                in(reg) b,
                options(pure, nomem, nostack)
            );
        }
        result
    }

    /// CPUIDを使ってプロセッサ情報を取得
    ///
    /// leaf=0: vendorID取得。
    /// rbxはLLVMが内部で使用するため、push/popで保護してから使う。
    pub fn cpuid_vendor() -> [u8; 12] {
        let (ebx, ecx, edx): (u32, u32, u32);
        unsafe {
            asm!(
                // rbxはLLVMが予約しているため自分で退避・復元する
                "push rbx",
                "cpuid",
                "mov {ebx_out:e}, ebx",
                "pop rbx",
                in("eax") 0_u32,
                ebx_out = out(reg) ebx,
                out("ecx") ecx,
                out("edx") edx,
                // stackを使うのでnostackは付けない
            );
        }
        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
        vendor
    }

    /// RDTSCで現在のタイムスタンプカウンターを読む
    ///
    /// プロファイリングやベンチマークに使う低レベルタイマー。
    pub fn rdtsc() -> u64 {
        let lo: u32;
        let hi: u32;
        unsafe {
            asm!(
                "rdtsc",
                out("eax") lo,
                out("edx") hi,
                options(nostack, nomem)
            );
        }
        ((hi as u64) << 32) | (lo as u64)
    }

    /// メモリバリア命令（フェンス）
    ///
    /// CPUの命令リオーダーを防ぐ。マルチコア同期で使う。
    pub fn memory_fence() {
        unsafe {
            asm!("mfence", options(nostack));
        }
    }

    /// ストアフェンス
    pub fn store_fence() {
        unsafe {
            asm!("sfence", options(nostack));
        }
    }

    /// ロードフェンス
    pub fn load_fence() {
        unsafe {
            asm!("lfence", options(nostack));
        }
    }

    /// PAUSE命令（スピンループ最適化）
    ///
    /// スピンロック内で呼ぶとCPUパイプラインの無駄を減らせる。
    pub fn cpu_pause() {
        unsafe {
            asm!("pause", options(nostack, nomem, preserves_flags));
        }
    }

    /// BSF（Bit Scan Forward）で最下位セットビットの位置を取得
    pub fn bit_scan_forward(x: u64) -> Option<u32> {
        if x == 0 {
            return None;
        }
        let result: u64;
        unsafe {
            asm!(
                "bsf {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        Some(result as u32)
    }

    /// BSR（Bit Scan Reverse）で最上位セットビットの位置を取得
    pub fn bit_scan_reverse(x: u64) -> Option<u32> {
        if x == 0 {
            return None;
        }
        let result: u64;
        unsafe {
            asm!(
                "bsr {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        Some(result as u32)
    }

    /// POPCNT（ポピュレーションカウント）でセットビット数を数える
    pub fn popcnt(x: u64) -> u32 {
        let result: u64;
        unsafe {
            asm!(
                "popcnt {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        result as u32
    }

    /// LZCNT（Leading Zero Count）で先頭ゼロビット数を数える
    pub fn lzcnt(x: u64) -> u32 {
        let result: u64;
        unsafe {
            asm!(
                "lzcnt {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        result as u32
    }

    /// inout(reg)でレジスタを入出力両方に使う例
    pub fn increment_asm(x: u64) -> u64 {
        let mut val = x;
        unsafe {
            asm!(
                "add {0}, 1",
                inout(reg) val,
                options(pure, nomem, nostack)
            );
        }
        val
    }

    /// const オペランド: コンパイル時定数をアセンブリに埋め込む
    pub fn shift_left_by<const N: u32>(x: u32) -> u32 {
        let mut val = x;
        unsafe {
            // {0}はval(inout)、{1}はN(const即値)
            asm!(
                "shl {0:e}, {1}",
                inout(reg) val,
                const N,
                options(nomem, nostack)
            );
        }
        val
    }

    /// 複数の命令を1つのasm!ブロックで書く
    ///
    /// 改行区切りまたはセミコロン区切りで複数命令を記述できる。
    pub fn multiply_add(a: u64, b: u64, c: u64) -> u64 {
        let result: u64;
        unsafe {
            asm!(
                "imul {tmp}, {b}",
                "add {tmp}, {c}",
                tmp = inout(reg) a => result,
                b = in(reg) b,
                c = in(reg) c,
                options(pure, nomem, nostack)
            );
        }
        result
    }
}

// ─── aarch64 固有の実装 ─────────────────────────────────────────────────────

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    use std::arch::asm;

    /// NOP命令
    pub fn nop() {
        unsafe {
            asm!("nop", options(nostack, nomem, preserves_flags));
        }
    }

    /// 加算（デモ）
    pub fn add_asm(a: u64, b: u64) -> u64 {
        let result: u64;
        unsafe {
            asm!(
                "add {0}, {1}, {2}",
                out(reg) result,
                in(reg) a,
                in(reg) b,
                options(pure, nomem, nostack)
            );
        }
        result
    }

    /// DMB ISH: データメモリバリア（Inner Shareableドメイン）
    pub fn memory_barrier() {
        unsafe {
            asm!("dmb ish", options(nostack));
        }
    }

    /// ISB: 命令同期バリア
    pub fn instruction_barrier() {
        unsafe {
            asm!("isb", options(nostack));
        }
    }

    /// CLZ（Count Leading Zeros）
    pub fn count_leading_zeros(x: u64) -> u32 {
        let result: u64;
        unsafe {
            asm!(
                "clz {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        result as u32
    }

    /// RBIT（Reverse Bits）: ビット逆順
    pub fn reverse_bits(x: u64) -> u64 {
        let result: u64;
        unsafe {
            asm!(
                "rbit {0}, {1}",
                out(reg) result,
                in(reg) x,
                options(pure, nomem, nostack)
            );
        }
        result
    }
}

// ─── アーキテクチャ非依存のラッパー ─────────────────────────────────────────

/// NOPを実行する（プラットフォーム対応）
pub fn nop() {
    #[cfg(target_arch = "x86_64")]
    x86_64::nop();

    #[cfg(target_arch = "aarch64")]
    aarch64::nop();

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // 他のアーキテクチャではRustのヒントを使う
        std::hint::spin_loop();
    }
}

/// メモリフェンスを実行する（プラットフォーム対応）
pub fn memory_fence() {
    #[cfg(target_arch = "x86_64")]
    x86_64::memory_fence();

    #[cfg(target_arch = "aarch64")]
    aarch64::memory_barrier();

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // 標準ライブラリのアトミックフェンスで代替
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop() {
        nop(); // パニックしなければOK
    }

    #[test]
    fn test_memory_fence() {
        memory_fence(); // パニックしなければOK
    }

    #[cfg(target_arch = "x86_64")]
    mod x86_tests {
        use super::super::x86_64::*;

        #[test]
        fn test_add_asm() {
            assert_eq!(add_asm(3, 4), 7);
            assert_eq!(add_asm(0, 0), 0);
            assert_eq!(add_asm(100, 200), 300);
        }

        #[test]
        fn test_increment_asm() {
            assert_eq!(increment_asm(0), 1);
            assert_eq!(increment_asm(41), 42);
        }

        #[test]
        fn test_multiply_add() {
            // a * b + c
            assert_eq!(multiply_add(3, 4, 5), 17); // 3*4+5
            assert_eq!(multiply_add(2, 3, 0), 6);
        }

        #[test]
        fn test_rdtsc_monotonic() {
            let t1 = rdtsc();
            let t2 = rdtsc();
            assert!(t2 >= t1, "RDTSCは単調増加すべき");
        }

        #[test]
        fn test_bit_scan_forward() {
            assert_eq!(bit_scan_forward(0b1010), Some(1));
            assert_eq!(bit_scan_forward(0b1000), Some(3));
            assert_eq!(bit_scan_forward(1), Some(0));
            assert_eq!(bit_scan_forward(0), None);
        }

        #[test]
        fn test_bit_scan_reverse() {
            assert_eq!(bit_scan_reverse(0b1010), Some(3));
            assert_eq!(bit_scan_reverse(1), Some(0));
            assert_eq!(bit_scan_reverse(0), None);
        }

        #[test]
        fn test_popcnt() {
            assert_eq!(popcnt(0), 0);
            assert_eq!(popcnt(0b1010), 2);
            assert_eq!(popcnt(u64::MAX), 64);
        }

        #[test]
        fn test_shift_left() {
            assert_eq!(shift_left_by::<1>(4), 8);
            assert_eq!(shift_left_by::<3>(1), 8);
        }

        #[test]
        fn test_cpuid_vendor() {
            let vendor = cpuid_vendor();
            // GenuineIntelかAuthenticAMDのどちらかが典型的
            let s = std::str::from_utf8(&vendor).unwrap_or("unknown");
            assert!(s.len() == 12, "ベンダーIDは12バイト: {}", s);
        }
    }

    #[cfg(target_arch = "aarch64")]
    mod aarch64_tests {
        use super::super::aarch64::*;

        #[test]
        fn test_add_asm() {
            assert_eq!(add_asm(3, 4), 7);
        }

        #[test]
        fn test_count_leading_zeros() {
            assert_eq!(count_leading_zeros(1), 63);
            assert_eq!(count_leading_zeros(u64::MAX), 0);
        }

        #[test]
        fn test_reverse_bits() {
            let x = 1_u64;
            let r = reverse_bits(x);
            assert_eq!(r, 1_u64 << 63);
        }
    }
}
