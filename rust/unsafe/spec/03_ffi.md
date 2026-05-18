# 外部関数インターフェース（FFI）

## 概要

FFIはRustから他言語（主にC）の関数を呼び出す、またはRustのコードを他言語から呼び出す機能です。

## C関数の呼び出し

```rust
extern "C" {
    fn abs(x: i32) -> i32;
    fn strlen(s: *const std::os::raw::c_char) -> usize;
}

// 呼び出しはunsafe
let result = unsafe { abs(-42) };
```

## Cからエクスポート

```rust
// Cから呼び出せるRust関数
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
```

## CStringとCStr

```rust
use std::ffi::{CString, CStr};

// RustのStringからCStringへ
let cstring = CString::new("hello").unwrap();
let ptr: *const i8 = cstring.as_ptr();

// CのポインタからCStrへ（unsafe）
let cstr = unsafe { CStr::from_ptr(ptr) };
let rust_str = cstr.to_str().unwrap();
```

## C ABIの型

| Rust | C |
|---|---|
| `std::os::raw::c_char` | `char` |
| `std::os::raw::c_int` | `int` |
| `std::os::raw::c_uint` | `unsigned int` |
| `std::os::raw::c_long` | `long` |
| `std::os::raw::c_void` | `void` |
| `*const T` | `const T*` |
| `*mut T` | `T*` |

## #[repr(C)] 構造体

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}
// Cの struct { double x; double y; } と同じレイアウト
```

## コールバック

```rust
// C ABIのコールバック型
type Callback = extern "C" fn(i32) -> i32;

// Rustで定義したCコールバック
extern "C" fn double_it(x: i32) -> i32 { x * 2 }

// Cの関数に渡す
some_c_function(double_it);
```

## 呼び出し規約

| 文字列 | 規約 |
|---|---|
| `"C"` | プラットフォームのデフォルトC ABI |
| `"stdcall"` | Windows x86の標準（Win32 API） |
| `"fastcall"` | レジスタ渡しを優先 |
| `"system"` | Windowsでは`"stdcall"`、他では`"C"` |
| `"Rust"` | Rustのデフォルト（安定していない） |

## 実装ファイル

`src/ffi.rs`
