extern crate nom;
pub use nom::bytes::complete::tag;
pub use nom::IResult;
use std::error::Error;

fn parse_input(input: &str) -> IResult<&str, &str> {
    //  note that this is really creating a function, the parser for abc
    //  vvvvv
    //         which is then called here, returning an IResult<&str, &str>
    //         vvvvv
    tag("abc")(input)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (leftover_input, output) = parse_input("abcWorld")?;
    assert_eq!(leftover_input, "World");
    assert_eq!(output, "abc");
    dbg!(leftover_input);
    dbg!(output);

    assert!(parse_input("defWorld").is_err());
    Ok(())
}
