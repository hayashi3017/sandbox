fn main() {
    let ex1 = [0u32; 8];
    dbg!(ex1.as_ptr());

    unsafe {
        let pointer = *ex1.as_ptr();
        dbg!(pointer);
    }

    let slice: &[u32] = &ex1;
    // スライスを (ポインタ, 長さ) に変換
    let (ptr, len): (*const u8, u32) = unsafe { std::mem::transmute(slice) };
    println!("Pointer: {:p}", ptr);
    println!("Length: {:?}", len);
}
