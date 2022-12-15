use std::collections::HashSet;

use aoc::parser::read_from_stdin_and_parse;

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
    fn projection(&self, y: i64) -> impl Iterator<Item = Pos> {
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

        let reach = (self_x - beacon_x).abs() + (self_y - beacon_y).abs();
        let distance = (y - self_y).abs();

        ((distance - reach + self_x)..=(reach - distance + self_x)).map(move |x| Pos { x, y })
    }
}

pub fn main() {
    let sensors = read_from_stdin_and_parse(parser::parse_input).unwrap();

    let beacons = sensors
        .iter()
        .map(|Sensor { beacon, .. }| *beacon)
        .collect::<HashSet<Pos>>();

    let y = 2000000;

    let coverage = sensors
        .iter()
        .flat_map(|s| s.projection(y))
        .collect::<HashSet<Pos>>();

    let n = coverage.difference(&beacons).count();

    println!("{}", n);
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
