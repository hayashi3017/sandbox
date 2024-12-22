use std::mem::ManuallyDrop;

fn main() {
    let ex1 = String::new();
    dbg!(ex1.as_ptr());

    let mut ex2 = String::new();
    let ex2_ptr = ex2.as_ptr();
    ex2.push_str("foo");
    dbg!(ex2_ptr);
    dbg!(ex2.as_ptr());
    // pushするとポインタが変わる？？
    assert_ne!(ex2.as_ptr(), ex2_ptr);

    debug_inner_string(&ex2);

    // ポインタの参照先をみる
    unsafe {
        let first_char = *ex2.as_ptr();
        println!("First byte of the string: {}", first_char);
        let char = first_char as char;
        dbg!(char);
    }
}

fn debug_inner_string(val: &String) {
    let s = ManuallyDrop::new(val);
    let ptr = s.as_ptr();
    let len = s.len();
    let capacity = s.capacity();

    println!("Pointer: {:p}", ptr);
    println!("Length: {}", len);
    println!("Capacity: {}", capacity);
}
