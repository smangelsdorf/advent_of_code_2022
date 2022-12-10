use std::collections::HashSet;
use std::io::Read;

use aoc::parser::nom_parse_to_owned;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Position {
    x: i64,
    y: i64,
}

impl Position {
    fn step(&mut self, direction: Direction) {
        use Direction::*;

        match direction {
            Up => self.y += 1,
            Down => self.y -= 1,
            Right => self.x += 1,
            Left => self.x -= 1,
        }
    }

    fn follow(&mut self, other: &Position) {
        let vec = Position {
            x: other.x - self.x,
            y: other.y - self.y,
        };

        if i64::max(vec.x.abs(), vec.y.abs()) >= 2 {
            // Ensure to only move one space at a time.
            self.x += vec.x.clamp(-1, 1);
            self.y += vec.y.clamp(-1, 1);
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Rope {
    parts: Vec<Position>,
}

impl Rope {
    fn new(parts: usize) -> Rope {
        assert!(parts >= 2);
        let parts = std::iter::repeat(Position::default()).take(parts).collect();
        Rope { parts }
    }

    fn step(&mut self, direction: Direction) {
        let mut iter = self.parts.iter_mut();
        let mut prev = {
            let head: &mut Position = iter.next().unwrap();
            head.step(direction);
            head.clone()
        };

        for part in iter {
            part.follow(&prev);
            prev = *part;
        }
    }

    fn tail(&self) -> Position {
        *self.parts.iter().last().unwrap()
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Move {
    direction: Direction,
    count: u64,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;

    let moves = nom_parse_to_owned(parser::input_parser, &buffer)?;

    let mut rope = Rope::new(10);
    let mut visited: HashSet<Position> = HashSet::default();

    for Move { direction, count } in moves {
        for _i in 0..count {
            rope.step(direction);
            visited.insert(rope.tail());
        }
    }

    println!("{}", visited.len());

    Ok(())
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space1};
    use nom::multi::separated_list1;
    use nom::sequence::separated_pair;
    use nom::{IResult, Parser};

    fn direction(input: &str) -> IResult<&str, Direction> {
        alt((
            tag("U").map(|_| Direction::Up),
            tag("D").map(|_| Direction::Down),
            tag("L").map(|_| Direction::Left),
            tag("R").map(|_| Direction::Right),
        ))
        .parse(input)
    }

    fn a_move(input: &str) -> IResult<&str, Move> {
        separated_pair(direction, space1, base10_numeric)
            .map(|(direction, count)| Move { direction, count })
            .parse(input)
    }

    pub(super) fn input_parser(input: &str) -> IResult<&str, Vec<Move>> {
        separated_list1(line_ending, a_move).parse(input)
    }
}
