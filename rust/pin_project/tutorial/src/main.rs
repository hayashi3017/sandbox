use std::pin::Pin;

use pin_project::pin_project;

fn main() {
    let mut foo = Box::pin(MyStruct::new());
    foo.as_mut().set_field(5);
    assert_eq!(foo.as_ref().field(), 5);
}

// #[pin_project]
struct MyStruct {
    field: usize,
}

impl MyStruct {
    fn new() -> Self {
        MyStruct { field: 0 }
    }

    fn field(self: Pin<&Self>) -> usize {
        self.field
    }

    fn set_field(mut self: Pin<&mut Self>, val: usize) {
        // let this = self.project();
        // *this.field = val;
        self.field = val;
    }
}

// https://tech-blog.optim.co.jp/entry/2020/03/05/160000#Pin%E3%81%AE%E4%B8%AD%E8%BA%AB%E3%82%92%E5%A4%89%E6%9B%B4%E3%81%99%E3%82%8B
