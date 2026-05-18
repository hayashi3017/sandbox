//! # ユニオン型（`union`）
//!
//! ユニオンはすべてのフィールドが同じメモリ領域を共有する型です。
//! Cの`union`に相当します。
//!
//! ## 特徴
//!
//! - フィールドへのアクセスはunsafe（どのフィールドが有効か不明）
//! - フィールドは`Copy`型または`ManuallyDrop<T>`でなければならない
//! - ドロップは実装されない（`Copy`型なので不要）
//! - `#[repr(C)]`でCレイアウトを保証できる
//!
//! ## 主な用途
//!
//! - Cの`union`型とのFFI
//! - タグ付きユニオン（Rustの`enum`の内部実装に近い）
//! - 型パンニング（`transmute`の代替）
//! - NaN-boxing などの高度な最適化

use std::mem;

// ─── 基本的なユニオン ───────────────────────────────────────────────────────

/// i32とf32が同じ4バイトを共有するユニオン
#[repr(C)]
union IntOrFloat {
    int_val: i32,
    float_val: f32,
}

/// f32のビットパターンをi32として読む（型パンニング）
///
/// これは`transmute`と同等だが、ユニオンによる方法はより明示的。
pub fn float_bits_as_int(f: f32) -> i32 {
    let u = IntOrFloat { float_val: f };
    // SAFETY: float_valで初期化したので4バイトは有効
    unsafe { u.int_val }
}

/// i32のビットパターンをf32として読む
pub fn int_bits_as_float(i: i32) -> f32 {
    let u = IntOrFloat { int_val: i };
    // SAFETY: int_valで初期化したので4バイトは有効
    unsafe { u.float_val }
}

// ─── Cとのインターフェースに使うユニオン ────────────────────────────────────

/// CのIPv4/IPv6アドレスユニオンに相当
#[repr(C)]
union IpAddress {
    v4: [u8; 4],
    v6: [u8; 16],
    raw: u128,
}

impl IpAddress {
    pub fn from_v4(octets: [u8; 4]) -> Self {
        IpAddress { v4: octets }
    }

    pub fn v4_octets(&self) -> [u8; 4] {
        // SAFETY: from_v4で初期化した場合のみ呼ぶこと
        unsafe { self.v4 }
    }

    pub fn as_u128(&self) -> u128 {
        // SAFETY: unionのrawフィールドはv4/v6と同じメモリ
        unsafe { self.raw }
    }
}

// ─── タグ付きユニオン ───────────────────────────────────────────────────────

/// タグ付きユニオン（discriminated union / tagged union）
///
/// Rustの`enum`はこれを安全に実装したもの。
/// ここでは手動で実装して仕組みを理解する。
#[repr(C)]
union ValueData {
    int: i64,
    float: f64,
    boolean: bool,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueTag {
    Int = 0,
    Float = 1,
    Bool = 2,
}

/// タグ付きユニオン構造体
#[repr(C)]
pub struct TaggedValue {
    tag: ValueTag,
    data: ValueData,
}

impl TaggedValue {
    pub fn from_int(i: i64) -> Self {
        TaggedValue {
            tag: ValueTag::Int,
            data: ValueData { int: i },
        }
    }

    pub fn from_float(f: f64) -> Self {
        TaggedValue {
            tag: ValueTag::Float,
            data: ValueData { float: f },
        }
    }

    pub fn from_bool(b: bool) -> Self {
        TaggedValue {
            tag: ValueTag::Bool,
            data: ValueData { boolean: b },
        }
    }

    pub fn tag(&self) -> ValueTag {
        self.tag
    }

    pub fn as_int(&self) -> Option<i64> {
        if self.tag == ValueTag::Int {
            // SAFETY: タグがIntの場合はintフィールドが有効
            Some(unsafe { self.data.int })
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if self.tag == ValueTag::Float {
            // SAFETY: タグがFloatの場合はfloatフィールドが有効
            Some(unsafe { self.data.float })
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.tag == ValueTag::Bool {
            // SAFETY: タグがBoolの場合はbooleanフィールドが有効
            Some(unsafe { self.data.boolean })
        } else {
            None
        }
    }
}

// ─── ManuallyDropを使うユニオン（非Copyフィールド）───────────────────────────

use std::mem::ManuallyDrop;

/// Stringを含むユニオン（ManuallyDropが必要）
///
/// DropするフィールドをManuallyDropで包むことでunsafe unionに格納できる。
/// ただしドロップは自分で管理する必要がある。
union StringOrBytes {
    string: ManuallyDrop<String>,
    bytes: ManuallyDrop<Vec<u8>>,
}

pub enum TextData {
    String(String),
    Bytes(Vec<u8>),
}

/// StringOrBytesユニオンを使った変換（教育目的）
pub fn string_as_bytes_parts(s: String) -> (usize, usize) {
    let len = s.len();
    let cap = s.capacity();
    let u = StringOrBytes {
        string: ManuallyDrop::new(s),
    };
    // SAFETY: stringフィールドで初期化した直後
    let recovered = unsafe { ManuallyDrop::into_inner(u.string) };
    drop(recovered);
    (len, cap)
}

// ─── ユニオンのサイズはフィールドの最大サイズ ─────────────────────────────

pub fn union_size_demonstration() -> (usize, usize, usize) {
    #[repr(C)]
    union Mixed {
        byte: u8,
        word: u16,
        dword: u32,
    }
    (
        mem::size_of::<Mixed>(),  // 4 (最大フィールドのサイズ)
        mem::size_of::<u8>(),
        mem::size_of::<u32>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_bits_as_int() {
        // f32の1.0はIEEE 754で 0x3F800000
        let bits = float_bits_as_int(1.0_f32);
        assert_eq!(bits, 0x3F800000_u32 as i32);
    }

    #[test]
    fn test_int_bits_as_float() {
        let f = int_bits_as_float(0x3F800000_u32 as i32);
        assert_eq!(f, 1.0_f32);
    }

    #[test]
    fn test_roundtrip_float_int() {
        let original = 3.14_f32;
        let bits = float_bits_as_int(original);
        let recovered = int_bits_as_float(bits);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_ip_address_v4() {
        let ip = IpAddress::from_v4([192, 168, 1, 1]);
        let octets = ip.v4_octets();
        assert_eq!(octets, [192, 168, 1, 1]);
    }

    #[test]
    fn test_tagged_value_int() {
        let v = TaggedValue::from_int(42);
        assert_eq!(v.tag(), ValueTag::Int);
        assert_eq!(v.as_int(), Some(42));
        assert_eq!(v.as_float(), None);
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn test_tagged_value_float() {
        let v = TaggedValue::from_float(3.14);
        assert_eq!(v.tag(), ValueTag::Float);
        assert!(v.as_float().is_some());
        assert!((v.as_float().unwrap() - 3.14).abs() < 1e-10);
        assert_eq!(v.as_int(), None);
    }

    #[test]
    fn test_tagged_value_bool() {
        let v = TaggedValue::from_bool(true);
        assert_eq!(v.tag(), ValueTag::Bool);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_union_size() {
        let (mixed, byte, dword) = union_size_demonstration();
        assert_eq!(mixed, dword); // ユニオンのサイズ == 最大フィールドのサイズ
        assert!(mixed > byte);
    }

    #[test]
    fn test_string_as_bytes_parts() {
        let s = String::from("hello");
        let (len, cap) = string_as_bytes_parts(s);
        assert_eq!(len, 5);
        assert!(cap >= 5);
    }
}
