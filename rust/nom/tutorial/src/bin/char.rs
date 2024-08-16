extern crate nom;
pub use nom::character::complete::alpha0;
pub use nom::IResult;
use nom::{
    character::complete::{alpha1, multispace0},
    Parser,
};

fn parser<'src, F, O, E>(f: F) -> impl Fn(&'src str) -> IResult<&'src str, O>
where
    F: Fn(&'src str) -> IResult<&'src str, O>,
    F: Parser<&'src str, O, E>,
    E: nom::error::ParseError<&'src str>,
{
    move |input| f(input)
}

fn main() {
    let input = "abc123def";
    let (remaining, letters) = parser(alpha0)(input).unwrap();
    assert_eq!(remaining, "123def");
    assert_eq!(letters, "abc");

    let input = "abc123def";
    let (remaining, letters) = parser(alpha1)(input).unwrap();
    assert_eq!(remaining, "123def");
    assert_eq!(letters, "abc");

    let input = " abc ";
    // let (remaining, letters) = multispace0::<&str, Error<&str>>(input).unwrap();
    let (remaining, letters) = parser(multispace0)(input).unwrap();
    assert_eq!(remaining, "abc ");
    assert_eq!(letters, " ");
}
