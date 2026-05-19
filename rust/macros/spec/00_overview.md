# Rustマクロ 学習ガイド

## 概要

このプロジェクトはRustのマクロ機能を網羅した学習用ライブラリです。

## マクロの2大分類

```
Rustのマクロ
├── 宣言的マクロ (Declarative Macros)
│   └── macro_rules! — パターンマッチングで展開
│
└── 手続きマクロ (Procedural Macros)
    ├── #[derive(...)]    — トレイト実装の自動生成
    ├── 属性マクロ        — #[attr] でアイテムを変換
    └── 関数形式マクロ   — name!() でコードを生成
```

## プロジェクト構成

```
rust/macros/
├── Cargo.toml          (ワークスペース)
├── learning/           (学習用クレート)
│   └── src/
│       ├── basics.rs           (基礎構文)
│       ├── repetition.rs       (繰り返し)
│       ├── fragments.rs        (フラグメント指定子)
│       ├── hygiene.rs          (衛生性)
│       ├── recursive.rs        (再帰・TTマンチャー)
│       ├── patterns.rs         (高度なパターン)
│       ├── builtin_macros.rs   (組み込みマクロ)
│       ├── proc_derive.rs      (deriveマクロの使用)
│       ├── proc_attribute.rs   (属性マクロの使用)
│       └── proc_function_like.rs (関数形式マクロの使用)
├── macros_impl/        (手続きマクロ実装クレート)
│   └── src/lib.rs      (Describe, Builder, logged, sql!, make_enum!)
└── spec/               (仕様ドキュメント)
```

## 実行方法

```bash
# テスト実行
cargo test

# 特定モジュールのテスト
cargo test -p macros-learning basics::

# ドキュメント生成
cargo doc --open
```

## 仕様書一覧

| ファイル | 内容 |
|---|---|
| `01_basics.md` | macro_rules!の基礎構文 |
| `02_repetition.md` | 繰り返し演算子 |
| `03_fragments.md` | フラグメント指定子 |
| `04_hygiene.md` | マクロ衛生性と$crate |
| `05_recursive.md` | 再帰マクロとTTマンチャー |
| `06_patterns.md` | 高度なパターン |
| `07_builtin.md` | 標準ライブラリ組み込みマクロ |
| `08_proc_derive.md` | Deriveマクロ |
| `09_proc_attribute.md` | 属性マクロ |
| `10_proc_function_like.md` | 関数形式マクロ |
