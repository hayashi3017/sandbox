# ユニオン型（`union`）

## 概要

ユニオンはすべてのフィールドが同じメモリ領域を共有する型です。Cの`union`に相当します。

## 基本構文

```rust
union MyUnion {
    int_val: i32,
    float_val: f32,
}

let u = MyUnion { int_val: 42 };

// フィールドアクセスはunsafe
let val = unsafe { u.int_val };
```

## 制約

- フィールドの型は`Copy`か`ManuallyDrop<T>`でなければならない
- ドロップは自動では行われない（Copyなので）
- `ManuallyDrop`フィールドは手動でdropを管理する

## 型パンニング（Type Punning）

```rust
union FloatInt {
    f: f32,
    i: u32,
}

let u = FloatInt { f: 1.0_f32 };
let bits = unsafe { u.i }; // 0x3F800000
```

これは`mem::transmute`の代替として使える。

## タグ付きユニオン

```rust
union Data {
    int: i64,
    float: f64,
}

struct TaggedValue {
    tag: u8,  // 0=int, 1=float
    data: Data,
}
```

Rustの`enum`はこれを安全に実装したもの。

## #[repr(C)]

```rust
#[repr(C)]
union CUnion {
    byte: u8,
    word: u16,
    dword: u32,
}
// Cの union { uint8_t byte; uint16_t word; uint32_t dword; } と同じ
```

## サイズ

ユニオンのサイズは最大フィールドのサイズ（+アライメントのパディング）:

```rust
union U { a: u8, b: u32 }
assert_eq!(std::mem::size_of::<U>(), 4); // u32のサイズ
```

## ManuallyDropを使った非Copyフィールド

```rust
use std::mem::ManuallyDrop;

union StringOrVec {
    string: ManuallyDrop<String>,
    vec: ManuallyDrop<Vec<u8>>,
}

let u = StringOrVec { string: ManuallyDrop::new(String::from("hi")) };
// 使い終わったら手動でdrop
unsafe { ManuallyDrop::drop(&mut /* ... */); }
```

## 実装ファイル

`src/unions.rs`
