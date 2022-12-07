use std::collections::BTreeMap;
use std::io::Read;
use std::iter::from_fn;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char, digit1, line_ending, space0, space1};
use nom::combinator::eof;
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{IResult, Parser};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
struct Crate(char);

#[derive(Eq, PartialEq, Debug)]
struct Ship {
    stacks: BTreeMap<u64, Vec<Crate>>,
}

impl Ship {
    fn perform(&mut self, m: Move) {
        let n = usize::try_from(m.count).unwrap();
        // Can't mutably take two values (stacks) from the map, so we need to store the moved
        // crates here in the interim.
        let mut v = Vec::with_capacity(n);

        if let Some(from) = self.stacks.get_mut(&m.from) {
            v.extend(from_fn(|| from.pop()).take(n));
        }

        if let Some(to) = self.stacks.get_mut(&m.to) {
            to.extend(v.into_iter().rev());
        }
    }

    fn tops(&self) -> impl Iterator<Item = char> + '_ {
        self.stacks.values().filter_map(|v| v.last()).map(|c| c.0)
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Move {
    count: u64,
    from: u64,
    to: u64,
}

pub fn main() {
    use parser::parse_input;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let (_blank, (mut ship, moves)) = parse_input(&input).unwrap();

    for m in moves {
        ship.perform(m);
    }

    println!("{}", ship.tops().collect::<String>());
}

mod parser {
    use super::*;

    fn base10_u64(input: &str) -> IResult<&str, u64> {
        digit1.map(|s| u64::from_str(s).unwrap()).parse(input)
    }

    fn air(input: &str) -> IResult<&str, Option<Crate>> {
        tag("   ").map(|_| None).parse(input)
    }

    fn krate(input: &str) -> IResult<&str, Option<Crate>> {
        delimited(char('['), anychar, char(']'))
            .map(|c| Some(Crate(c)))
            .parse(input)
    }

    fn names(input: &str) -> IResult<&str, Vec<u64>> {
        terminated(
            delimited(space0, separated_list1(space1, base10_u64), space0),
            pair(line_ending, line_ending),
        )
        .parse(input)
    }

    // This is a flip+transpose+zip operation which unwraps the Option values and drops the None
    // values. It's a little on the dense side, read with caution.
    fn collate_stacks(rows: Vec<Vec<Option<Crate>>>, names: Vec<u64>) -> Ship {
        // Reverse the rows, turn it into an iterator over iterators.
        let mut row_iters = rows
            .into_iter()
            .rev()
            .map(|i| i.into_iter())
            .collect::<Vec<_>>();

        // Create a new iterator which takes one pass over row_iters each time. This is the
        // transpose step.
        let stacks = from_fn(move || {
            let i = row_iters.iter_mut().filter_map(|i| i.next()).peekable();

            // Build the vectors, unwrapping Option and discarding None.
            Some(i.filter_map(std::convert::identity).collect::<Vec<_>>())
                // Empty list means we're finished and should return None from the iterator.
                .filter(|v| !v.is_empty())
        });

        // Build the map.
        let stacks = names.iter().copied().zip(stacks).collect();
        Ship { stacks }
    }

    fn ship(input: &str) -> IResult<&str, Ship> {
        pair(
            many1(terminated(
                separated_list1(char(' '), alt((air, krate))),
                line_ending,
            )),
            names,
        )
        .map(|(stacks, names)| collate_stacks(stacks, names))
        .parse(input)
    }

    fn moves(input: &str) -> IResult<&str, Vec<Move>> {
        separated_list1(
            line_ending,
            tuple((
                preceded(tuple((tag("move"), space1)), base10_u64),
                preceded(tuple((space1, tag("from"), space1)), base10_u64),
                preceded(tuple((space1, tag("to"), space1)), base10_u64),
            ))
            .map(|(count, from, to)| Move { count, from, to }),
        )
        .parse(input)
    }

    fn end_of_input(input: &str) -> IResult<&str, ()> {
        terminated(many0(line_ending).map(|_| ()), eof).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, (Ship, Vec<Move>)> {
        terminated(tuple((ship, moves)), end_of_input).parse(input)
    }

    #[cfg(test)]
    pub(super) use self::moves as parse_moves;
    #[cfg(test)]
    pub(super) use self::ship as parse_ship;
}

#[cfg(test)]
mod tests {
    use super::parser::*;

    #[test]
    fn test_parse() {
        let input = "[A] [B]     [C] [D]\n\
                     [E] [F] [G] [H] [I]\n\
                      1   2   3   4   5\n\n";

        let (_, ship) = parse_ship(input).unwrap();

        assert_eq!(
            ship,
            Ship {
                names: vec![1, 2, 3, 4, 5],
                stacks: [
                    (1, vec![Crate('E'), Crate('A')]),
                    (2, vec![Crate('F'), Crate('B')]),
                    (3, vec![Crate('G')]),
                    (4, vec![Crate('H'), Crate('C')]),
                    (5, vec![Crate('I'), Crate('D')]),
                ]
                .into()
            }
        );

        let input = "move 3 from 2 to 1\n\
                     move 2 from 1 to 4\n\
                     move 6 from 0 to 100\n";

        let (_, moves) = parse_moves(input).unwrap();

        assert_eq!(
            moves,
            vec![
                Move {
                    count: 3,
                    from: 2,
                    to: 1
                },
                Move {
                    count: 2,
                    from: 1,
                    to: 4
                },
                Move {
                    count: 6,
                    from: 0,
                    to: 100
                },
            ]
        );
    }
}
