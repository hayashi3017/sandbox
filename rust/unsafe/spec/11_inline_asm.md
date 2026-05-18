# インラインアセンブリ（`asm!` / `global_asm!`）

## 概要

Rustのアセンブリ直接記述機能。アーキテクチャ固有命令やレジスタを使う場合に使います。  
Rust 1.59以降でstableに昇格しました。

## 基本構文

```rust
use std::arch::asm;

unsafe {
    asm!(
        "命令テンプレート",
        // オペランド,
        // オプション
    );
}
```

## オペランド制約

| 制約 | 方向 | 説明 |
|---|---|---|
| `in(reg) val` | 入力 | 汎用レジスタ |
| `out(reg) val` | 出力 | 汎用レジスタ |
| `inout(reg) a => b` | 入出力 | 同じレジスタを入力と出力に使う |
| `in("rax") val` | 入力 | 特定レジスタを指定 |
| `lateout(reg) val` | 遅延出力 | 入力の後で書き込まれる |
| `const N` | 定数 | コンパイル時定数を即値として使う |
| `sym SYMBOL` | シンボル | 関数や静的変数のアドレス |

## レジスタクラス（x86_64）

| クラス | レジスタ |
|---|---|
| `reg` | rax, rbx, rcx, rdx, rsi, rdi, r8-r15 |
| `reg_byte` | al, bl, cl, dl |
| `xmm_reg` | xmm0-xmm15 |
| `ymm_reg` | ymm0-ymm15 |
| `zmm_reg` | zmm0-zmm31 |

## オプション

| オプション | 意味 |
|---|---|
| `nostack` | スタックを変更しない（赤ゾーンを使わない） |
| `nomem` | メモリを読み書きしない |
| `pure` | 副作用なし（同じ入力→同じ出力） |
| `preserves_flags` | フラグレジスタを変更しない |
| `att_syntax` | AT&T構文（デフォルトはIntel構文） |
| `raw` | テンプレート置換なし |

## 例: x86_64

```rust
// NOP
unsafe { asm!("nop", options(nostack, nomem, preserves_flags)); }

// 加算
let result: u64;
unsafe {
    asm!(
        "add {0}, {1}",
        inout(reg) a => result,
        in(reg) b,
        options(pure, nomem, nostack)
    );
}

// RDTSC（タイムスタンプカウンター）
let (lo, hi): (u32, u32);
unsafe {
    asm!(
        "rdtsc",
        out("eax") lo,
        out("edx") hi,
        options(nostack, nomem)
    );
}
let tsc: u64 = ((hi as u64) << 32) | lo as u64;
```

## 複数命令

```rust
unsafe {
    asm!(
        "mov {tmp}, {x}",  // 改行で区切る
        "shl {tmp}, 1",
        "add {x}, {tmp}",
        x = inout(reg) x,
        tmp = out(reg) _,  // _ = 使わない出力
        options(pure, nomem, nostack)
    );
}
```

## global_asm!

```rust
// 関数の外でアセンブリを記述
std::arch::global_asm!(
    ".globl my_asm_function",
    "my_asm_function:",
    "mov rax, 42",
    "ret",
);
```

## プラットフォーム対応

```rust
#[cfg(target_arch = "x86_64")]
fn optimized() { /* x86_64実装 */ }

#[cfg(target_arch = "aarch64")]
fn optimized() { /* ARM実装 */ }

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn optimized() { /* fallback */ }
```

## 実装ファイル

`src/inline_asm.rs`
