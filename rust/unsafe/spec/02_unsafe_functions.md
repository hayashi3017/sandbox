# unsafe関数の定義と呼び出し

## 概要

`unsafe fn`は「呼び出し元が特定の不変条件を守る責任を持つ」という契約です。  
コンパイラは安全条件を検証できないため、プログラマが責任を持ちます。

## 構文

```rust
// unsafe関数の定義
unsafe fn dangerous(ptr: *const i32) -> i32 {
    *ptr  // unsafeブロックは不要（関数自体がunsafe）
}

// 呼び出し
unsafe {
    let x = 42;
    let result = dangerous(&x as *const i32);
}
```

## ドキュメントのパターン

```rust
/// 説明
///
/// # Safety
///
/// - 条件1
/// - 条件2
pub unsafe fn my_function(ptr: *const i32) -> i32 {
    // ...
}
```

## unsafeブロックの最小化

```rust
// 悪い例: 必要以上に広いunsafeブロック
pub fn bad_example(data: &[i32]) -> i32 {
    unsafe {
        let result = *data.as_ptr();  // unsafe操作
        let doubled = result * 2;     // これはsafe
        doubled
    }
}

// 良い例: unsafeは必要な行だけ
pub fn good_example(data: &[i32]) -> i32 {
    // SAFETY: dataが非空なので先頭ポインタは有効
    let result = unsafe { *data.as_ptr() };
    result * 2  // unsafeブロック外
}
```

## 安全なラッパーパターン

```rust
// 内部でunsafeを使うが、公開APIはsafe
pub fn get_first(slice: &[i32]) -> Option<i32> {
    if slice.is_empty() {
        return None;
    }
    // SAFETY: スライスが非空なので有効
    Some(unsafe { *slice.as_ptr() })
}
```

## SAFETY コメントの規約

```rust
// SAFETY: [根拠をここに書く]
unsafe { ... }
```

`# Safety`はドキュメント（公開API向け）、`// SAFETY:`はインラインコメント（実装内）で使い分ける。

## 実装ファイル

`src/unsafe_functions.rs`
