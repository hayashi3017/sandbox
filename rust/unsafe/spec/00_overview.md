# Rust unsafe コード 学習ガイド

## 概要

このプロジェクトはRustの`unsafe`コードの主要概念と機能を網羅した学習用ライブラリです。  
各モジュールには実装例・安全条件のドキュメント・テストが含まれます。

## なぜ unsafe が存在するか

Rustの安全性保証は強力ですが、以下の場面では表現できない正しいプログラムがあります:

- C言語との連携（FFI）
- OSやハードウェアのプリミティブなAPI
- `Vec`や`Arc`のような標準ライブラリの実装自体
- パフォーマンスが極めてクリティカルな低レベル操作

`unsafe`はこれらのユースケースのための「脱出ハッチ」です。

## unsafe が許可する5つの操作

| 操作 | 説明 |
|---|---|
| 生ポインタのderef | `*const T`/`*mut T`を通じて値を読み書き |
| unsafe関数の呼び出し | `unsafe fn`や`extern "C" fn`の呼び出し |
| 可変静的変数のアクセス | `static mut`の読み書き |
| unsafeトレイトの実装 | `unsafe impl Foo for Bar {}` |
| unionフィールドへのアクセス | どのフィールドが有効かはプログラマが判断 |

## モジュール一覧

| モジュール | ファイル | 仕様書 |
|---|---|---|
| 生ポインタ | `src/raw_pointers.rs` | `spec/01_raw_pointers.md` |
| unsafe関数 | `src/unsafe_functions.rs` | `spec/02_unsafe_functions.md` |
| FFI | `src/ffi.rs` | `spec/03_ffi.md` |
| static mut | `src/static_mut.rs` | `spec/04_static_mut.md` |
| union | `src/unions.rs` | `spec/05_unions.md` |
| transmute | `src/transmute_ops.rs` | `spec/06_transmute.md` |
| スライス操作 | `src/slice_ops.rs` | `spec/07_slice_ops.md` |
| メモリ管理 | `src/memory_management.rs` | `spec/08_memory_management.md` |
| ポインタ演算 | `src/pointer_arithmetic.rs` | `spec/09_pointer_arithmetic.md` |
| unsafeトレイト | `src/unsafe_traits.rs` | `spec/10_unsafe_traits.md` |
| インラインアセンブリ | `src/inline_asm.rs` | `spec/11_inline_asm.md` |

## 実行方法

```bash
# ライブラリのテストを実行
cargo test

# デモバイナリを実行
cargo run --bin demo

# ドキュメントを生成
cargo doc --open
```

## 安全コードとの対比

```rust
// safe: 借用チェッカーが保証
let x = 42;
let r = &x;
println!("{}", r);

// unsafe: プログラマが保証
let x = 42_i32;
let raw: *const i32 = &x;
let val = unsafe { *raw }; // ptrが有効であることをプログラマが保証
println!("{}", val);
```

## unsafeコードを書く際の鉄則

1. **unsafeブロックを最小化する**: unsafeが必要な行だけをブロックに含める
2. **# Safety ドキュメントを必ず書く**: `unsafe fn`には呼び出し条件を明記
3. **安全なAPIでラップする**: 内部実装にunsafeを閉じ込め、公開APIはsafeにする
4. **不変条件をコードに近い場所にコメントする**: SAFETY: コメントで根拠を説明
5. **Miriで検証する**: `cargo +nightly miri test`で未定義動作を検出

## 参考資料

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — unsafe Rustの公式ガイド
- [Unsafe Code Guidelines Reference](https://rust-lang.github.io/unsafe-code-guidelines/)
- [std::ptr ドキュメント](https://doc.rust-lang.org/std/ptr/)
