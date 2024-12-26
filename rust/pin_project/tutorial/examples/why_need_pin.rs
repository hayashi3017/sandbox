fn main() {
    let foo = SelfRef::new();
    assert_ne!(&foo.x as *const _, foo.ptr);

    dbg!(&foo.x as *const _ as usize);
    dbg!(foo.ptr as usize);
}

struct SelfRef {
    x: usize,
    ptr: *const usize,
}

impl SelfRef {
    fn new() -> SelfRef {
        let mut this = SelfRef {
            x: 0,
            ptr: std::ptr::null(),
        };
        this.ptr = &this.x;

        assert_eq!(&this.x as *const _, this.ptr);
        dbg!(&this.x as *const _ as usize);
        dbg!(this.ptr as usize);
        // ここでmoveするのでxのアドレスが変わる
        this
    }
}
