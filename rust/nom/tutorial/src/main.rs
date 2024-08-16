use tutorial::parser::length_value;

fn main() {
    let input = &b"\x00\x03abcefg"[..];
    let length = length_value(input).unwrap();
    println!("input: {:?}, length: {:?}", input, length);
}
