use aoc::parser::read_from_stdin_and_parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Void,
    Ground,
    Wall,
}

#[derive(Debug, PartialEq, Eq)]
struct Map {
    map: Vec<Vec<Tile>>,
}

impl Map {
    fn initial_position(&self) -> Position {
        // First ground tile in the first row
        let x = self
            .map
            .get(0)
            .unwrap()
            .iter()
            .position(|&tile| tile == Tile::Ground)
            .unwrap() as isize;

        Position { x, y: 0 }
    }

    fn next_position(&self, position: Position, facing: Facing) -> Position {
        // Step, and wrap around if we hit the edge.
        let (x, y) = match facing {
            Facing::North => (
                position.x,
                (position.y - 1).rem_euclid(self.map.len() as isize),
            ),
            Facing::East => (
                (position.x + 1).rem_euclid(self.map[position.y as usize].len() as isize),
                position.y,
            ),
            Facing::South => (
                position.x,
                (position.y + 1).rem_euclid(self.map.len() as isize),
            ),
            Facing::West => (
                (position.x - 1).rem_euclid(self.map[position.y as usize].len() as isize),
                position.y,
            ),
        };

        Position { x, y }
    }

    fn advance_position(&self, original_position: Position, facing: Facing) -> Position {
        // Iterate "next_position" until we hit a ground or wall. If it's a wall
        // we return the original position, if it's ground we return that position.
        let mut position = original_position;
        loop {
            let next_position = self.next_position(position, facing);

            match self
                .map
                .get(next_position.y as usize)
                .and_then(|row| row.get(next_position.x as usize))
                .unwrap_or(&Tile::Void)
            {
                Tile::Ground => return next_position,
                Tile::Wall => return original_position,
                Tile::Void => position = next_position,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Move(u64),
    TurnLeft,
    TurnRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Facing {
    East = 0,
    South = 1,
    West = 2,
    North = 3,
}

impl std::ops::Add<Instruction> for Facing {
    type Output = Self;

    fn add(self, rhs: Instruction) -> Self::Output {
        match rhs {
            Instruction::TurnLeft => match self {
                Facing::North => Facing::West,
                Facing::East => Facing::North,
                Facing::South => Facing::East,
                Facing::West => Facing::South,
            },
            Instruction::TurnRight => match self {
                Facing::North => Facing::East,
                Facing::East => Facing::South,
                Facing::South => Facing::West,
                Facing::West => Facing::North,
            },
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
    position: Position,
    facing: Facing,
}

impl State {
    fn initial(map: &Map) -> Self {
        Self {
            position: map.initial_position(),
            facing: Facing::East,
        }
    }
}

pub fn main() {
    let (map, instructions) = read_from_stdin_and_parse(parser::parse_input).unwrap();

    let mut state = State::initial(&map);
    for instruction in instructions {
        match instruction {
            Instruction::Move(distance) => {
                for _ in 0..distance {
                    state.position = map.advance_position(state.position, state.facing);
                }
            }
            _ => state.facing = state.facing + instruction,
        }
    }

    let n = 4 * (state.position.x + 1) + 1000 * (state.position.y + 1) + state.facing as isize;
    println!("{:?} - {}", state, n);
}

mod parser {
    use super::*;
    use aoc::parser::base10_numeric;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::line_ending,
        combinator::{eof, map, value},
        multi::{many0, many1, separated_list1},
        sequence::{separated_pair, terminated, tuple},
        IResult, Parser,
    };

    fn tile(input: &str) -> IResult<&str, Tile> {
        alt((
            value(Tile::Wall, tag("#")),
            value(Tile::Ground, tag(".")),
            value(Tile::Void, tag(" ")),
        ))
        .parse(input)
    }

    fn monkey_map(input: &str) -> IResult<&str, Map> {
        separated_list1(line_ending, many1(tile))
            .map(|map| Map { map })
            .parse(input)
    }

    fn instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
        many1(flip_flop(
            map(base10_numeric, Instruction::Move),
            alt((
                value(Instruction::TurnLeft, tag("L")),
                value(Instruction::TurnRight, tag("R")),
            )),
        ))
        .parse(input)
    }

    fn flip_flop<'a, T, P0, P1>(
        mut a: P0,
        mut b: P1,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, T> + 'a
    where
        P0: Parser<&'a str, T, nom::error::Error<&'a str>> + 'a,
        P1: Parser<&'a str, T, nom::error::Error<&'a str>> + 'a,
    {
        let mut flip = false;
        move |input| {
            flip = !flip;
            if flip {
                a.parse(input)
            } else {
                b.parse(input)
            }
        }
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, (Map, Vec<Instruction>)> {
        terminated(
            separated_pair(monkey_map, tuple((line_ending, line_ending)), instructions),
            tuple((many0(line_ending), eof)),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_input() {
            // Mix of wall, ground, and void tiles
            let input = "\
                #########\n\
                #...#...#\n\
                #####...#\n\
                #   #...#\n\
                #########\n\
                \n\
                14L10R5\n\
            ";

            let expected_map = Map {
                map: vec![
                    vec![
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                    ],
                    vec![
                        Tile::Wall,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Wall,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Wall,
                    ],
                    vec![
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Wall,
                    ],
                    vec![
                        Tile::Wall,
                        Tile::Void,
                        Tile::Void,
                        Tile::Void,
                        Tile::Wall,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Ground,
                        Tile::Wall,
                    ],
                    vec![
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                        Tile::Wall,
                    ],
                ],
            };

            let expected_instructions = vec![
                Instruction::Move(14),
                Instruction::TurnLeft,
                Instruction::Move(10),
                Instruction::TurnRight,
                Instruction::Move(5),
            ];

            let (input, (map, instructions)) = parse_input(input).unwrap();
            assert_eq!(input, "");
            assert_eq!(map, expected_map);
            assert_eq!(instructions, expected_instructions);
        }
    }
}
