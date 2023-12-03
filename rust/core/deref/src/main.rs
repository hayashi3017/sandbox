use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

struct Selector<T> {
    elements: Vec<T>,
    current: usize,
}

impl<T> Deref for Selector<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.elements[self.current]
    }
}

impl<T> DerefMut for Selector<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.elements[self.current]
    }
}

fn main() {
    println!("Hello, world!");
    let mut s = Selector {
        elements: vec!['x', 'y', 'z'],
        current: 2,
    };
    assert_eq!(*s, 'z');

    // 参照解決型変換
    assert!(s.is_alphabetic());

    *s = 'w';
    assert_eq!(s.elements, ['x', 'y', 'w']);

    let s2 = Selector {
        elements: vec!["good", "bad", "ugly"],
        current: 2,
    };
    fn show_it(thing: &str) {
        println!("{}", thing);
    }
    // 型の不整合を解決するために参照解決型変換が行われる
    show_it(&s2);

    fn show_it_generic<T: Display>(thing: T) {
        println!("{}", thing);
    }
    // 型パラメータの制約を満たすためには参照解決型変換が行われない
    // show_it_generic(&s2);
    show_it_generic(&*s2);
    show_it_generic(&s2 as &str);
}
