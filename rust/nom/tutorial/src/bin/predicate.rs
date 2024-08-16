extern crate nom;
use nom::bytes::complete::{take_till, take_until, take_while};
use nom::character::is_alphabetic;
use nom::sequence::terminated;
use nom::IResult;
use std::error::Error;

fn parser<'src, I: 'src + ?Sized, F, O>(f: F) -> impl Fn(&'src I) -> IResult<&'src I, O>
where
    F: Fn(&'src I) -> IResult<&'src I, O>,
{
    move |input| f(input)
}

fn parse_sentence(input: &str) -> IResult<&str, &str> {
    terminated(take_until("."), take_while(|c| c == '.' || c == ' '))(input)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (remaining, parsed) = parse_sentence("I am Tom. I write Rust.")?;
    assert_eq!(parsed, "I am Tom");
    assert_eq!(remaining, "I write Rust.");

    let parsing_error = parse_sentence("Not a sentence (no period at the end)");
    assert!(parsing_error.is_err());

    assert_eq!(
        parser(take_while(is_alphabetic))(&b"abc123"[..]),
        Ok((&b"123"[..], &b"abc"[..]))
    );

    assert_eq!(
        parser(take_till(is_alphabetic))(&b"123abc"[..]),
        Ok((&b"abc"[..], &b"123"[..]))
    );

    assert_eq!(
        parser(take_until("world"))(&b"Hello world"[..]),
        Ok((&b"world"[..], &b"Hello "[..]))
    );

    Ok(())
}
