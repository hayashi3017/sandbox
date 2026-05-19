//! # Rustマクロ学習ライブラリ
//!
//! このクレートはRustのマクロ機能を網羅した学習用ライブラリです。
//!
//! ## モジュール構成
//!
//! ### 宣言的マクロ（`macro_rules!`）
//!
//! | モジュール | 概要 |
//! |---|---|
//! | [`basics`] | 基本構文・パターン・アーム |
//! | [`repetition`] | 繰り返し（`$(...)*`, `$()+`, `$(...)?`） |
//! | [`fragments`] | フラグメント指定子（expr, ident, ty, tt, ...） |
//! | [`hygiene`] | マクロ衛生性と`$crate` |
//! | [`recursive`] | 再帰マクロとTTマンチャー |
//! | [`patterns`] | 高度なパターン（push-down accumulation, callbacks） |
//! | [`builtin_macros`] | 標準ライブラリ組み込みマクロ |
//!
//! ### 手続きマクロ（`macros_impl`クレート）
//!
//! | モジュール | 概要 |
//! |---|---|
//! | [`proc_derive`] | `#[derive(...)]`マクロ |
//! | [`proc_attribute`] | 属性マクロ（`#[attr]`） |
//! | [`proc_function_like`] | 関数形式マクロ（`macro!(...)`） |

pub mod basics;
pub mod builtin_macros;
pub mod fragments;
pub mod hygiene;
pub mod patterns;
pub mod proc_attribute;
pub mod proc_derive;
pub mod proc_function_like;
pub mod recursive;
pub mod repetition;
