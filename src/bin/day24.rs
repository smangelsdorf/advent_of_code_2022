// For some reason this works for my input, but not for the sample input.

use std::{collections::HashSet, time::Instant};

use aoc::parser::read_from_stdin_and_parse;
use rayon::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
}

#[derive(Debug, Clone)]
struct Field {
    blizzards: Vec<u8>,
    dims: (usize, usize),
    start_col: usize,
    end_col: usize,
}

impl Field {
    fn step(&self) -> Field {
        let (width, height) = self.dims;
        let mut out = self.clone();

        self.blizzards
            .par_iter()
            .enumerate()
            .map(|(i, _)| {
                let (x, y) = (i % width, i / width);

                let left = if x == 0 { width - 1 } else { x - 1 };
                let right = if x == width - 1 { 0 } else { x + 1 };
                let up = if y == 0 { height - 1 } else { y - 1 };
                let down = if y == height - 1 { 0 } else { y + 1 };

                self.blizzards[y * width + left] & Direction::Right as u8
                    | self.blizzards[y * width + right] & Direction::Left as u8
                    | self.blizzards[up * width + x] & Direction::Down as u8
                    | self.blizzards[down * width + x] & Direction::Up as u8
            })
            .collect_into_vec(&mut out.blizzards);

        out
    }

    fn is_clear(&self, (x, y): (usize, usize)) -> bool {
        let (width, _height) = self.dims;
        let i = y * width + x;

        self.blizzards[i] == 0
    }
}

struct State {
    positions: HashSet<(usize, usize)>,
    minute: u64,
    start: (usize, usize),
    target: (usize, usize),
}

impl State {
    fn update(self, field: &Field) -> State {
        let minute = self.minute;
        let (dim_x, dim_y) = field.dims;

        let positions: HashSet<(usize, usize)> = self
            .positions
            .iter()
            .flat_map(|&(x, y)| {
                let left = if x == 0 { None } else { Some((x - 1, y)) };
                let right = if x >= dim_x - 1 {
                    None
                } else {
                    Some((x + 1, y))
                };
                let up = if y == 0 { None } else { Some((x, y - 1)) };
                let down = if y >= dim_y - 1 {
                    None
                } else {
                    Some((x, y + 1))
                };

                [left, right, up, down, Some((x, y)), Some(self.start)]
                    .into_iter()
                    .flatten()
                    .filter_map(|position| {
                        Some(position).filter(|position| field.is_clear(*position))
                    })
            })
            .collect();

        State {
            positions,
            minute: minute + 1,
            ..self
        }
    }

    fn is_done(&self) -> bool {
        self.positions.contains(&self.target)
    }

    fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    fn reset_continue(self, start: (usize, usize), target: (usize, usize)) -> State {
        let mut positions = self.positions;
        positions.retain(|&pos| pos == start);
        let minute = self.minute + 2;

        State {
            positions,
            minute,
            start,
            target,
        }
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("positions", &self.positions)
            .field("minute", &self.minute)
            .field("start", &self.start)
            .field("target", &self.target)
            .finish()
    }
}

struct InfiniteFields {
    limit: usize,
    storage: Vec<Field>,
}

impl InfiniteFields {
    fn new(initial: Field) -> Self {
        fn lcm(a: u64, b: u64) -> u64 {
            (a * b) / gcd(a, b)
        }

        fn gcd(a: u64, b: u64) -> u64 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }

        let limit = lcm(initial.dims.0 as u64, initial.dims.1 as u64) as usize;
        let storage = std::iter::successors(Some(initial), |f| Some(f.step()))
            .take(limit)
            .collect();

        InfiniteFields { limit, storage }
    }

    fn get(&self, i: usize) -> &Field {
        &self.storage[i % self.limit]
    }

    fn initial_state(&self) -> Option<State> {
        self.storage
            .iter()
            .enumerate()
            .map(|(minute, field)| {
                let (_dim_x, dim_y) = field.dims;
                let start = (field.start_col, 0);
                let target = (field.end_col, dim_y - 1);

                let positions = Some(start)
                    .filter(|&pos| field.is_clear(pos))
                    .into_iter()
                    .collect();
                let minute = minute as u64;

                State {
                    positions,
                    minute,
                    start,
                    target,
                }
            })
            .find(|state| !state.positions.is_empty())
    }
}

pub fn main() {
    let field = read_from_stdin_and_parse(parser::parse_input).unwrap();
    let (_dim_x, dim_y) = field.dims;
    let start_col = field.start_col;
    let end_col = field.end_col;

    let start_pos = (start_col, 0);
    let end_pos = (end_col, dim_y - 1);

    let start = Instant::now();
    let fields = InfiniteFields::new(field);
    println!(
        "Building {} fields took {:?}",
        fields.limit,
        start.elapsed()
    );

    let start = Instant::now();
    let mut state = fields.initial_state().expect("initial state");

    println!("initial state: {:?}", state);

    while !state.is_done() {
        let field = fields.get(state.minute as usize + 1);
        state = state.update(field);

        if state.is_empty() {
            println!("empty state at minute {}", state.minute + 1);
            return;
        }
    }

    println!("Found initial solution at minute {}", state.minute + 1);
    println!("Took {:?}", start.elapsed());

    state = state.reset_continue(end_pos, start_pos);
    println!("first reset: {:?}", state);

    while !state.is_done() {
        let field = fields.get(state.minute as usize + 1);
        state = state.update(field);

        if state.is_empty() {
            println!("empty state at minute {}", state.minute + 1);
            return;
        }
    }

    println!("Found return solution at minute {}", state.minute + 1);
    println!("Running total {:?}", start.elapsed());

    state = state.reset_continue(start_pos, end_pos);
    println!("second reset: {:?}", state);

    while !state.is_done() {
        let field = fields.get(state.minute as usize + 1);
        state = state.update(field);

        if state.is_empty() {
            println!("empty state at minute {}", state.minute + 1);
            return;
        }
    }

    println!("Found final solution at minute {}", state.minute + 1);
    println!("Final total {:?}", start.elapsed());
}

mod parser {
    use super::*;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, line_ending},
        combinator::{eof, map, value},
        multi::{many0, many1, many1_count, separated_list1},
        sequence::{delimited, terminated, tuple},
        IResult, Parser,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Tile {
        Ground,
        Blizzard(Direction),
    }

    impl Tile {
        fn into_u8(self) -> u8 {
            match self {
                Tile::Ground => 0,
                Tile::Blizzard(direction) => direction as u8,
            }
        }
    }

    fn blizzard_direction(input: &str) -> IResult<&str, Direction> {
        alt((
            value(Direction::Up, tag("^")),
            value(Direction::Down, tag("v")),
            value(Direction::Left, tag("<")),
            value(Direction::Right, tag(">")),
        ))
        .parse(input)
    }

    fn tile(input: &str) -> IResult<&str, Tile> {
        alt((
            value(Tile::Ground, char('.')),
            map(blizzard_direction, Tile::Blizzard),
        ))
        .parse(input)
    }

    fn start_end_line(input: &str) -> IResult<&str, usize> {
        terminated(many1_count(tag("#")), tuple((tag("."), many1(tag("#")))))
            .map(|i| i - 1)
            .parse(input)
    }

    fn grid_line(input: &str) -> IResult<&str, Vec<Tile>> {
        delimited(tag("#"), many1(tile), tag("#")).parse(input)
    }

    fn grid_lines(input: &str) -> IResult<&str, Vec<Vec<Tile>>> {
        separated_list1(line_ending, grid_line).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Field> {
        tuple((
            terminated(start_end_line, line_ending),
            terminated(grid_lines, line_ending),
            terminated(start_end_line, tuple((many0(line_ending), eof))),
        ))
        .map(|(start_col, grid, end_col)| {
            let (width, height) = (grid[0].len(), grid.len());

            let blizzards = grid
                .into_iter()
                .flat_map(|line| line.into_iter().map(Tile::into_u8))
                .collect();

            Field {
                blizzards,
                dims: (width, height),
                start_col,
                end_col,
            }
        })
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        let input = "\
            #.#####\n\
            #.....#\n\
            #>....#\n\
            #.....#\n\
            #...v.#\n\
            #.....#\n\
            #####.#";

        let (remaining, parsed) = parser::parse_input(input).unwrap();
        assert_eq!(remaining, "");

        let r = Direction::Right as u8;
        let d = Direction::Down as u8;

        #[rustfmt::skip]
        let expected = vec![
            0,0,0,0,0,
            r,0,0,0,0,
            0,0,0,0,0,
            0,0,0,d,0,
            0,0,0,0,0,
        ];

        assert_eq!(parsed.blizzards, expected);
        assert_eq!(parsed.dims, (5, 5));
        assert_eq!(parsed.start_col, 0);
        assert_eq!(parsed.end_col, 4);

        let field = parsed.step();

        #[rustfmt::skip]
        let expected = vec![
            0,0,0,0,0,
            0,r,0,0,0,
            0,0,0,0,0,
            0,0,0,0,0,
            0,0,0,d,0,
        ];

        assert_eq!(field.blizzards, expected);

        let field = field.step();

        #[rustfmt::skip]
        let expected = vec![
            0,0,0,d,0,
            0,0,r,0,0,
            0,0,0,0,0,
            0,0,0,0,0,
            0,0,0,0,0,
        ];

        assert_eq!(field.blizzards, expected);

        let field = field.step();

        #[rustfmt::skip]
        let expected = vec![
            0,0,0,0,0,
            0,0,0,r|d,0,
            0,0,0,0,0,
            0,0,0,0,0,
            0,0,0,0,0,
        ];

        assert_eq!(field.blizzards, expected);

        let field = field.step();

        #[rustfmt::skip]
        let expected = vec![
            0,0,0,0,0,
            0,0,0,0,r,
            0,0,0,d,0,
            0,0,0,0,0,
            0,0,0,0,0,
        ];

        assert_eq!(field.blizzards, expected);
    }
}
