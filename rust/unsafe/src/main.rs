/// デモ実行バイナリ
///
/// 各モジュールの主要機能をデモします。
/// `cargo run --bin demo` で実行。

use rust_unsafe::{
    ffi, inline_asm, memory_management, pointer_arithmetic, raw_pointers, slice_ops, static_mut,
    transmute_ops, unions, unsafe_functions, unsafe_traits,
};

fn main() {
    println!("=== Rust unsafe コード デモ ===\n");

    // 1. 生ポインタ
    println!("--- 1. 生ポインタ ---");
    let x = 42_i32;
    println!("read_via_raw_pointer: {}", raw_pointers::read_via_raw_pointer(&x));
    println!("null pointer is null: {}", raw_pointers::demonstrate_null_pointer());
    println!("safe_deref(&42): {:?}", raw_pointers::safe_deref(&42_i32 as *const i32));
    println!();

    // 2. unsafe関数
    println!("--- 2. unsafe関数 ---");
    println!("get_first([10,20,30]): {:?}", unsafe_functions::get_first(&[10, 20, 30]));
    println!("sum_via_pointer([1..5]): {}", unsafe_functions::sum_via_pointer(&[1, 2, 3, 4, 5]));
    println!();

    // 3. FFI
    println!("--- 3. FFI ---");
    println!("c_abs(-42): {}", ffi::c_abs(-42));
    println!("string_length_via_c(\"hello\"): {}", ffi::string_length_via_c("hello"));
    println!("use_callback(5): {}", ffi::use_callback(5));
    let p = ffi::Point::new(3.0, 4.0);
    println!("Point(3,4).distance: {}", p.distance_from_origin());
    println!();

    // 4. static mut
    println!("--- 4. static mut ---");
    unsafe {
        static_mut::reset_counter();
        static_mut::increment_counter();
        static_mut::increment_counter();
        println!("counter after 2 increments: {}", static_mut::get_counter());
        static_mut::reset_counter();
    }
    static_mut::atomic_increment();
    static_mut::atomic_increment();
    println!("atomic counter: {}", static_mut::atomic_get());
    static_mut::atomic_reset();
    println!();

    // 5. union
    println!("--- 5. union ---");
    let bits = unions::float_bits_as_int(1.0_f32);
    println!("f32(1.0) bits as i32: 0x{:08X}", bits as u32);
    let v = unions::TaggedValue::from_int(100);
    println!("TaggedValue::Int(100).as_int(): {:?}", v.as_int());
    println!();

    // 6. transmute
    println!("--- 6. transmute ---");
    let (t, b) = transmute_ops::compare_approaches_f32(1.0);
    println!("f32(1.0) via transmute: 0x{:08X}", t);
    println!("f32(1.0) via to_bits:   0x{:08X}", b);
    println!();

    // 7. slice ops
    println!("--- 7. スライス操作 ---");
    let data = [1_i32, 2, 3, 4, 5];
    let s = unsafe { slice_ops::slice_from_ptr(data.as_ptr(), data.len()) };
    println!("slice_from_ptr: {:?}", s);
    let bytes = b"Rust\xe8\xa8\x80\xe8\xaa\x9e"; // "Rust言語"
    println!("bytes_to_str_safe: {:?}", slice_ops::bytes_to_str_safe(bytes));
    println!();

    // 8. メモリ管理
    println!("--- 8. メモリ管理 ---");
    println!("box_raw_roundtrip(10): {}", memory_management::box_raw_roundtrip(10));
    println!("write_to_uninitialized: {}", memory_management::write_to_uninitialized());
    println!("manual_alloc_demo: {}", memory_management::manual_alloc_demo());
    println!("init_fixed_array: {:?}", memory_management::init_fixed_array());
    println!();

    // 9. ポインタ演算
    println!("--- 9. ポインタ演算 ---");
    let nums = [1_i32, 2, 3, 4, 5];
    println!("sum_with_ptr_arithmetic: {}", pointer_arithmetic::sum_with_pointer_arithmetic(&nums));
    let sorted_data = [1, 3, 5, 7, 9, 11];
    println!("binary_search_raw(7): {:?}", pointer_arithmetic::binary_search_raw(&sorted_data, 7));
    let iter_result: Vec<i32> = pointer_arithmetic::RawSliceIter::new(&nums).collect();
    println!("RawSliceIter: {:?}", iter_result);
    println!();

    // 10. unsafeトレイト
    println!("--- 10. unsafeトレイト ---");
    let n = 42_i32;
    let bytes_of_n = unsafe_traits::PlainOldData::as_bytes(&n);
    println!("i32(42) as bytes: {:?}", bytes_of_n);
    let mut mb = unsafe_traits::MyBox::new(999_i32);
    println!("MyBox<i32>::get(): {}", mb.get());
    *mb.get_mut() = 1000;
    println!("MyBox<i32> after mutation: {}", mb.get());
    println!();

    // 11. インラインアセンブリ
    println!("--- 11. インラインアセンブリ ---");
    inline_asm::nop();
    println!("nop() 実行完了");
    inline_asm::memory_fence();
    println!("memory_fence() 実行完了");

    #[cfg(target_arch = "x86_64")]
    {
        let t1 = inline_asm::x86_64::rdtsc();
        let t2 = inline_asm::x86_64::rdtsc();
        println!("RDTSC monotonic: {} <= {} = {}", t1, t2, t1 <= t2);
        println!("popcnt(0xFF): {}", inline_asm::x86_64::popcnt(0xFF));
        println!("bit_scan_forward(0b1100): {:?}", inline_asm::x86_64::bit_scan_forward(0b1100));
    }

    println!("\n=== デモ完了 ===");
}
