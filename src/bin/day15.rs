use aoc::parser::read_from_stdin_and_parse;
use itertools::Itertools;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
struct Pos {
    x: i64,
    y: i64,
}

#[derive(PartialEq, Eq, Debug)]
struct Sensor {
    pos: Pos,
    beacon: Pos,
}

impl Sensor {
    fn projection(&self, y: i64) -> Option<(i64, i64)> {
        let Sensor {
            pos: Pos {
                x: self_x,
                y: self_y,
            },
            beacon: Pos {
                x: beacon_x,
                y: beacon_y,
            },
        } = self;

        // The distance from the sensor to its beacon is the known reach of the sensor.
        let reach = (self_x - beacon_x).abs() + (self_y - beacon_y).abs();
        // The distance consumed by moving to the target y coordinate.
        let distance = (y - self_y).abs();

        // We can use the remaining reach to spread left/right from there, as long as
        // we haven't already exceeded our reach.
        Some(((distance - reach + self_x), (reach - distance + self_x))).filter(|(a, b)| a <= b)
    }
}

// Collapse overlapping ranges into one, or returns both ranges as Err.
//
// The ranges are considered inclusive, so (1, 2) and (3, 4) would collapse to (1, 4)
fn collapse(
    (a0, b0): (i64, i64),
    (a1, b1): (i64, i64),
) -> Result<(i64, i64), ((i64, i64), (i64, i64))> {
    if (a0..=(b0 + 1)).contains(&a1) || (a1..=(b1 + 1)).contains(&a0) {
        Ok((i64::min(a0, a1), i64::max(b0, b1)))
    } else {
        Err(((a0, b0), (a1, b1)))
    }
}

pub fn main() {
    let sensors = read_from_stdin_and_parse(parser::parse_input).unwrap();

    let field_size = 4000000;

    let distress = (0..=field_size)
        .map(|y| {
            // Project each sensor into the current "row", sort the ranges and
            // collapse any that overlap.
            let v = sensors
                .iter()
                .flat_map(|s| s.projection(y))
                .sorted_by_key(|(a, _b)| *a)
                .coalesce(collapse)
                .collect::<Vec<_>>();
            (y, v)
        })
        // Any row that has a gap between ranges, or isn't fully covered at the
        // start/end, that's our beacon position. The problem promises only one
        // of these (So this grabs the first one, ignoring the possibility of
        // others).
        .filter_map(|(y, coverage)| match coverage.as_slice() {
            &[(a, _b)] if a > 0 => Some((0, y)),
            &[(_a, b)] if b < field_size => Some((field_size, y)),
            &[(_, b0), (a1, _)] if b0 + 1 < a1 => Some((b0 + 1, y)),
            _ => None,
        })
        .next();

    match distress {
        Some((x, y)) => println!("{}", x * 4000000 + y),
        None => println!("Nothing"),
    }
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space0};
    use nom::combinator::eof;
    use nom::multi::{many0, many1, separated_list1};
    use nom::sequence::{preceded, separated_pair, terminated, tuple};
    use nom::{IResult, Parser};

    fn pos(input: &str) -> IResult<&str, Pos> {
        separated_pair(
            preceded(tag("x="), base10_numeric),
            tuple((space0, tag(","), space0)),
            preceded(tag("y="), base10_numeric),
        )
        .map(|(x, y)| Pos { x, y })
        .parse(input)
    }

    fn sensor(input: &str) -> IResult<&str, Sensor> {
        tuple((
            preceded(tag("Sensor at "), pos),
            preceded(tag(": closest beacon is at "), pos),
        ))
        .map(|(pos, beacon)| Sensor { pos, beacon })
        .parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<Sensor>> {
        terminated(
            separated_list1(many1(line_ending), sensor),
            tuple((many0(line_ending), eof)),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_input() {
            let input = "\
                Sensor at x=2, y=18: closest beacon is at x=-2, y=15\n\
                Sensor at x=9, y=16: closest beacon is at x=10, y=16\n\
                Sensor at x=13, y=2: closest beacon is at x=15, y=3\n";

            let (_, sensors) = parse_input(input).unwrap();

            assert_eq!(
                sensors,
                vec![
                    Sensor {
                        pos: Pos { x: 2, y: 18 },
                        beacon: Pos { x: -2, y: 15 }
                    },
                    Sensor {
                        pos: Pos { x: 9, y: 16 },
                        beacon: Pos { x: 10, y: 16 }
                    },
                    Sensor {
                        pos: Pos { x: 13, y: 2 },
                        beacon: Pos { x: 15, y: 3 }
                    },
                ]
            );
        }
    }
}
