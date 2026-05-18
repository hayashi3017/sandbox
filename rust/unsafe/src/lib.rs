//! # Rust unsafe コード学習ライブラリ
//!
//! このクレートはRustの`unsafe`コードの主要な概念と機能を網羅した学習用ライブラリです。
//!
//! ## モジュール構成
//!
//! | モジュール | 概要 |
//! |---|---|
//! | [`raw_pointers`] | 生ポインタ（`*const T`, `*mut T`）の操作 |
//! | [`unsafe_functions`] | unsafe関数の定義と呼び出し |
//! | [`ffi`] | 外部関数インターフェース（C言語との連携） |
//! | [`static_mut`] | 可変静的変数 |
//! | [`unions`] | ユニオン型 |
//! | [`transmute_ops`] | `mem::transmute`によるメモリ再解釈 |
//! | [`slice_ops`] | 生ポインタからのスライス生成 |
//! | [`memory_management`] | 手動メモリ管理 |
//! | [`pointer_arithmetic`] | ポインタ演算 |
//! | [`unsafe_traits`] | unsafeトレイトの定義と実装 |
//! | [`inline_asm`] | インラインアセンブリ |

pub mod ffi;
pub mod inline_asm;
pub mod memory_management;
pub mod pointer_arithmetic;
pub mod raw_pointers;
pub mod slice_ops;
pub mod static_mut;
pub mod transmute_ops;
pub mod unions;
pub mod unsafe_functions;
pub mod unsafe_traits;
