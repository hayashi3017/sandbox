# ポインタ演算

## 概要

生ポインタのアドレス計算。スライスやカスタムコレクションの内部実装で使います。

## 主なメソッド

| メソッド | 引数 | 説明 |
|---|---|---|
| `ptr.add(n)` | `usize` | n要素分進む（UB: 範囲外） |
| `ptr.sub(n)` | `usize` | n要素分戻る（UB: 範囲外） |
| `ptr.offset(n)` | `isize` | n要素分移動（正負可） |
| `ptr.wrapping_add(n)` | `usize` | wrapあり（UBなし、derefは危険） |
| `ptr.wrapping_sub(n)` | `usize` | wrapあり（UBなし） |
| `ptr.byte_add(n)` | `usize` | nバイト分進む |
| `ptr.byte_sub(n)` | `usize` | nバイト分戻る |
| `ptr.offset_from(base)` | `*const T` | 要素数の差（`isize`） |
| `ptr.byte_offset_from(base)` | `*const T` | バイト数の差 |

## add / sub の安全条件

```rust
let data = [1_i32, 2, 3, 4, 5];
let ptr = data.as_ptr();

// SAFETY: data.len()以内（厳密には+1まで）
let end = unsafe { ptr.add(data.len()) }; // endは最終要素の1つ後ろ

// SAFETY: ptrはendより前
let second = unsafe { ptr.add(1) };
```

「1つ後ろ」のアドレス（`ptr.add(len)`）を計算することは許可されていますが、
dereferenceは未定義動作です。

## offset vs add/sub

```rust
// offsetはisizeを受け取る（前後に移動できる）
let ptr = data.as_ptr();
let elem = unsafe { ptr.offset(2) }; // 前進
let prev = unsafe { ptr.offset(-1) }; // 後退（境界内なら）
```

## wrapping_add の使いどころ

```rust
// アドレスの計算にUBなしで使える
// ただしderefすることはできない（アドレスが不正になり得るため）
let sentinel = ptr.wrapping_add(usize::MAX); // 決してderefしない
```

## offset_from

```rust
let data = [1, 2, 3, 4, 5];
let start = data.as_ptr();
let end = unsafe { start.add(5) };

// SAFETY: 同じアロケーション内
let count = unsafe { end.offset_from(start) }; // 5
let reverse = unsafe { start.offset_from(end) }; // -5
```

## ポインタを使ったイテレータ

```rust
struct RawIter<T> {
    ptr: *const T,
    end: *const T,
}

impl<T: Copy> Iterator for RawIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.ptr == self.end { return None; }
        let val = unsafe { *self.ptr };
        self.ptr = unsafe { self.ptr.add(1) };
        Some(val)
    }
}
```

## 実装ファイル

`src/pointer_arithmetic.rs`
