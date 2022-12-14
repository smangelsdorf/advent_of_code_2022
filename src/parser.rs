use std::io::Read;
use std::iter::Sum;
use std::str::FromStr;

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{map_res, opt, recognize};
use nom::sequence::preceded;
use nom::{Finish, IResult, Parser};

pub fn base10_numeric<N>(input: &str) -> IResult<&str, N>
where
    N: Sum<N> + FromStr,
{
    map_res(recognize(preceded(opt(tag("-")), digit1)), |s| {
        N::from_str(s)
    })
    .parse(input)
}

pub fn nom_error_to_owned<I>(e: nom::error::Error<&I>) -> nom::error::Error<I::Owned>
where
    I: ToOwned + ?Sized,
    I::Owned: 'static,
{
    let nom::error::Error { input, code } = e;
    nom::error::Error {
        input: input.to_owned(),
        code,
    }
}

// Lifetime hacks to make the `?` operator usable with nom results.
//
// This was more work than just pattern matching it.
pub fn nom_parse_to_owned<I, O, P>(
    mut parser: P,
    input: &I,
) -> Result<O, nom::error::Error<I::Owned>>
where
    I: ToOwned + ?Sized,
    I::Owned: 'static,
    P: for<'i> Parser<&'i I, O, nom::error::Error<&'i I>>,
{
    match parser.parse(input).finish() {
        Ok((_i, o)) => Ok(o),
        Err(e) => Err(nom_error_to_owned(e)),
    }
}

// So much parsing. Time for another shortcut.
pub fn read_from_stdin_and_parse<O, P>(parser: P) -> Result<O, Box<dyn std::error::Error>>
where
    P: for<'i> Parser<&'i str, O, nom::error::Error<&'i str>>,
{
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;

    let out = nom_parse_to_owned(parser, &buffer)?;
    Ok(out)
}
