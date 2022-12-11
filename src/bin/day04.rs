use std::io::BufRead;

use nom::{bytes::complete::tag, character::complete::digit1, combinator::map_res, IResult};

type Range = std::ops::RangeInclusive<u64>;
struct Ranges(Range, Range);

impl Ranges {
    fn overlapping(&self) -> bool {
        let Ranges(r0, r1) = self;

        (r0.contains(r1.start()) || r0.contains(r1.end()))
            || (r1.contains(r0.start()) || r1.contains(r0.end()))
    }
}

fn integer_parser(input: &str) -> IResult<&str, u64> {
    map_res(digit1, |s| u64::from_str_radix(s, 10))(input)
}

fn range_parser(input: &str) -> IResult<&str, Range> {
    let (input, a) = integer_parser(input)?;
    let (input, _) = tag("-")(input)?;
    let (input, b) = integer_parser(input)?;

    Ok((input, a..=b))
}

fn line_parser(input: &str) -> IResult<&str, Ranges> {
    let (input, a) = range_parser(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, b) = range_parser(input)?;

    Ok((input, Ranges(a, b)))
}

pub fn main() {
    let n: usize = std::io::stdin()
        .lock()
        .lines()
        .map(Result::unwrap)
        .map(|s| line_parser(&s).map(|(_input, r)| r).expect("valid parse"))
        .filter(Ranges::overlapping)
        .count();

    println!("{}", n);
}
