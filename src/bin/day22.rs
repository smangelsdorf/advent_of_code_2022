use std::collections::{BTreeSet, HashMap};

use aoc::parser::read_from_stdin_and_parse;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Void,
    Ground,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CubeSide {
    // Rubik's Cube notation
    F,
    B,
    U,
    D,
    L,
    R,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Region {
    x0y0: Position,
    xnyn: Position,
}

impl Region {
    fn contains(&self, pos: Position) -> bool {
        self.x0y0.x <= pos.x && self.xnyn.x >= pos.x && self.x0y0.y <= pos.y && self.xnyn.y >= pos.y
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CubeLayout {
    dimension: usize,
    regions: HashMap<CubeSide, Region>,

    // e.g. The first two entries would be:
    // (D, South) -> (F, North)
    // (F, North) -> (D, South)
    edges: HashMap<(CubeSide, Facing), (CubeSide, Facing)>,
    void_regions: Vec<Region>,
}

impl CubeLayout {
    fn infer(vec: &Vec<Vec<Tile>>) -> Result<CubeLayout, &'static str> {
        // I'm sure there's a better way to do this. I fumbled my way through
        // this intuitively and didn't look for a better algorithm.

        let all_lens = vec.iter().map(|row| row.len()).collect::<BTreeSet<_>>();

        let max_width = *all_lens.iter().max().ok_or("no lengths")?;
        let dimension = max_width.max(vec.len()) / 4;

        for len in std::iter::once(vec.len()).chain(all_lens.iter().copied()) {
            if len % dimension != 0 {
                return Err("unable to infer dimension");
            }
        }

        let (void_regions, cube_regions) = (0..vec.len() / dimension)
            .flat_map(|big_y| {
                (0..max_width / dimension).map(move |big_x| {
                    let x0y0 = Position {
                        x: big_x as isize * dimension as isize,
                        y: big_y as isize * dimension as isize,
                    };

                    let xnyn = Position {
                        x: x0y0.x + dimension as isize - 1,
                        y: x0y0.y + dimension as isize - 1,
                    };

                    Region { x0y0, xnyn }
                })
            })
            .partition::<Vec<_>, _>(|region| {
                vec.get(region.x0y0.y as usize)
                    .and_then(|row| row.get(region.x0y0.x as usize))
                    .copied()
                    .unwrap_or(Tile::Void)
                    == Tile::Void
            });

        let mut longest_vertical_line = cube_regions
            .iter()
            .copied()
            .into_group_map_by(|r| r.x0y0.x)
            .into_values()
            .max_by_key(Vec::len)
            .expect("couldn't find longest line");

        longest_vertical_line.sort_by_key(|r| r.x0y0.y);

        let sides = [CubeSide::D, CubeSide::F, CubeSide::U, CubeSide::B];
        let mut regions: HashMap<CubeSide, Region> = sides
            .iter()
            .copied()
            .zip(longest_vertical_line.iter().copied())
            .collect();

        let remaining = cube_regions
            .iter()
            .copied()
            .filter(|r| !regions.values().any(|v| v == r))
            .collect::<Vec<_>>();

        let ys = longest_vertical_line
            .iter()
            .map(|r| r.x0y0.y)
            .collect::<BTreeSet<_>>();

        let left = remaining
            .iter()
            .copied()
            .filter(|r| {
                ys.contains(&r.x0y0.y)
                    && r.x0y0.x == regions[&CubeSide::D].x0y0.x - dimension as isize
            })
            .collect::<Vec<_>>();

        let left = match left.as_slice() {
            &[region] => region,
            _ => return Err("unable to infer L"),
        };

        let right = remaining
            .iter()
            .copied()
            .filter(|r| {
                ys.contains(&r.x0y0.y)
                    && r.x0y0.x == regions[&CubeSide::D].x0y0.x + dimension as isize
            })
            .collect::<Vec<_>>();

        let right = match right.as_slice() {
            &[region] => region,
            _ => return Err("unable to infer R"),
        };

        regions.insert(CubeSide::L, left);
        regions.insert(CubeSide::R, right);

        let remaining_regions = cube_regions
            .iter()
            .copied()
            .filter(|r| !regions.values().any(|v| v == r))
            .collect::<Vec<_>>();

        let remaining_sides = [
            CubeSide::L,
            CubeSide::R,
            CubeSide::F,
            CubeSide::B,
            CubeSide::U,
            CubeSide::D,
        ]
        .into_iter()
        .filter(|k| !regions.contains_key(k))
        .collect::<Vec<_>>();

        let remaining = match remaining_regions.as_slice() {
            [remaining] => remaining,
            _ => return Err("unable to infer remaining region (not implemented, didn't need it)"),
        };

        let remaining_side = match remaining_sides.as_slice() {
            [remaining] => remaining,
            _ => return Err("unable to infer remaining side (not implemented, didn't need it)"),
        };

        regions.insert(*remaining_side, *remaining);

        let mut edges: HashMap<(CubeSide, Facing), (CubeSide, Facing)> = regions
            .iter()
            .tuple_combinations()
            .filter_map(|((&ka, &a), (&kb, &b))| {
                // Figure out which edges are connected
                if a.x0y0.x == b.x0y0.x {
                    if a.x0y0.y == b.x0y0.y + dimension as isize {
                        Some(((ka, Facing::North), (kb, Facing::South)))
                    } else if a.x0y0.y + dimension as isize == b.x0y0.y {
                        Some(((ka, Facing::South), (kb, Facing::North)))
                    } else {
                        None
                    }
                } else if a.x0y0.y == b.x0y0.y {
                    if a.x0y0.x == b.x0y0.x + dimension as isize {
                        Some(((ka, Facing::West), (kb, Facing::East)))
                    } else if a.x0y0.x + dimension as isize == b.x0y0.x {
                        Some(((ka, Facing::East), (kb, Facing::West)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flat_map(|(a, b)| [(a, b), (b, a)])
            .collect();

        // 2-3 should be enough to figure out the rest, but 5 is guaranteed.
        for _ in 0..5 {
            for face in [
                CubeSide::L,
                CubeSide::R,
                CubeSide::F,
                CubeSide::B,
                CubeSide::U,
                CubeSide::D,
            ] {
                for facing in [Facing::North, Facing::East, Facing::South, Facing::West] {
                    if !edges.contains_key(&(face, facing)) {
                        let left = edges
                            .get(&(face, facing + Instruction::TurnLeft))
                            .and_then(|&(side, facing)| {
                                edges.get(&(side, facing + Instruction::TurnLeft))
                            })
                            .map(|&(side, facing)| (side, facing + Instruction::TurnLeft));

                        let right = edges
                            .get(&(face, facing + Instruction::TurnRight))
                            .and_then(|&(side, facing)| {
                                edges.get(&(side, facing + Instruction::TurnRight))
                            })
                            .map(|&(side, facing)| (side, facing + Instruction::TurnRight));

                        if let Some((new_face, new_facing)) = left.or(right) {
                            edges.insert((face, facing), (new_face, new_facing));
                        }
                    }
                }
            }
        }

        if edges.len() != 24 {
            return Err("unable to infer all edges");
        }

        Ok(Self {
            dimension,
            regions,
            edges,
            void_regions,
        })
    }

    fn next_position(&self, position: Position, facing: Facing) -> (Position, Facing) {
        let (&current_side, &current_region) = self
            .regions
            .iter()
            .find(|(_, region)| region.contains(position))
            .unwrap();

        // Step, and wrap around if we hit the edge.
        let (x, y) = match facing {
            Facing::North => (position.x, position.y - 1),
            Facing::East => (position.x + 1, position.y),
            Facing::South => (position.x, position.y + 1),
            Facing::West => (position.x - 1, position.y),
        };

        let pos = Position { x, y };

        if current_region.contains(pos) {
            return (pos, facing);
        }

        let (new_side, new_side_edge) = self
            .edges
            .get(&(current_side, facing))
            .unwrap_or_else(|| panic!("no edge for {:?} {:?}", current_region, facing));

        let new_region = self.regions.get(new_side).unwrap();

        let offset = match facing {
            Facing::North => position.x - current_region.x0y0.x,
            Facing::East => position.y - current_region.x0y0.y,
            Facing::South => current_region.xnyn.x - position.x,
            Facing::West => current_region.xnyn.y - position.y,
        };

        match new_side_edge {
            Facing::North => (
                Position {
                    x: new_region.xnyn.x - offset,
                    y: new_region.x0y0.y,
                },
                Facing::South,
            ),
            Facing::East => (
                Position {
                    x: new_region.xnyn.x,
                    y: new_region.xnyn.y - offset,
                },
                Facing::West,
            ),
            Facing::South => (
                Position {
                    x: new_region.x0y0.x + offset,
                    y: new_region.xnyn.y,
                },
                Facing::North,
            ),
            Facing::West => (
                Position {
                    x: new_region.x0y0.x,
                    y: new_region.x0y0.y + offset,
                },
                Facing::East,
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Map {
    map: Vec<Vec<Tile>>,
    cube_layout: CubeLayout,
}

impl Map {
    fn new(map: Vec<Vec<Tile>>) -> Self {
        let cube_layout = CubeLayout::infer(&map).expect("unable to infer cube layout");
        Self { map, cube_layout }
    }

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

    fn advance_position(
        &self,
        original_position: Position,
        original_facing: Facing,
    ) -> (Position, Facing) {
        // Iterate "next_position" until we hit a ground or wall. If it's a wall
        // we return the original position, if it's ground we return that position.
        let mut position = original_position;
        let mut facing = original_facing;
        loop {
            let (next_position, next_facing) = self.cube_layout.next_position(position, facing);

            match self
                .map
                .get(next_position.y as usize)
                .and_then(|row| row.get(next_position.x as usize))
                .unwrap_or(&Tile::Void)
            {
                Tile::Ground => return (next_position, next_facing),
                Tile::Wall => return (original_position, original_facing),
                Tile::Void => (position, facing) = (next_position, next_facing),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    CubeLayout::infer(&map.map).unwrap();

    let mut state = State::initial(&map);
    for instruction in instructions {
        match instruction {
            Instruction::Move(distance) => {
                for _ in 0..distance {
                    (state.position, state.facing) =
                        map.advance_position(state.position, state.facing);
                }
            }
            _ => state.facing = state.facing + instruction,
        }
    }

    let n = 4 * (state.position.x + 1) + 1000 * (state.position.y + 1) + state.facing as isize;
    println!("{:?} - {}", state, n);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_layout_infer() {
        let input = r"
  ..
  ..
....
....
  ....
  ....
    ..
    ..

0R5R2R8L1"
            .strip_prefix("\n")
            .unwrap();

        let (map, instructions) = parser::parse_input(input).unwrap().1;

        let expected = [
            (
                CubeSide::D,
                Region {
                    x0y0: Position { x: 2, y: 0 },
                    xnyn: Position { x: 3, y: 1 },
                },
            ),
            (
                CubeSide::F,
                Region {
                    x0y0: Position { x: 2, y: 2 },
                    xnyn: Position { x: 3, y: 3 },
                },
            ),
            (
                CubeSide::U,
                Region {
                    x0y0: Position { x: 2, y: 4 },
                    xnyn: Position { x: 3, y: 5 },
                },
            ),
            (
                CubeSide::L,
                Region {
                    x0y0: Position { x: 0, y: 2 },
                    xnyn: Position { x: 1, y: 3 },
                },
            ),
            (
                CubeSide::R,
                Region {
                    x0y0: Position { x: 4, y: 4 },
                    xnyn: Position { x: 5, y: 5 },
                },
            ),
            (
                CubeSide::B,
                Region {
                    x0y0: Position { x: 4, y: 6 },
                    xnyn: Position { x: 5, y: 7 },
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<CubeSide, Region>>();

        assert_eq!(map.cube_layout.regions, expected);

        let expected_edges = [
            ((CubeSide::D, Facing::South), (CubeSide::F, Facing::North)),
            ((CubeSide::D, Facing::West), (CubeSide::L, Facing::North)),
            ((CubeSide::D, Facing::East), (CubeSide::R, Facing::East)),
            ((CubeSide::D, Facing::North), (CubeSide::B, Facing::East)),
            ((CubeSide::F, Facing::South), (CubeSide::U, Facing::North)),
            ((CubeSide::F, Facing::West), (CubeSide::L, Facing::East)),
            ((CubeSide::F, Facing::East), (CubeSide::R, Facing::North)),
            ((CubeSide::F, Facing::North), (CubeSide::D, Facing::South)),
            ((CubeSide::U, Facing::South), (CubeSide::B, Facing::West)),
            ((CubeSide::U, Facing::West), (CubeSide::L, Facing::South)),
            ((CubeSide::U, Facing::East), (CubeSide::R, Facing::West)),
            ((CubeSide::U, Facing::North), (CubeSide::F, Facing::South)),
            ((CubeSide::L, Facing::South), (CubeSide::U, Facing::West)),
            ((CubeSide::L, Facing::West), (CubeSide::B, Facing::South)),
            ((CubeSide::L, Facing::East), (CubeSide::F, Facing::West)),
            ((CubeSide::L, Facing::North), (CubeSide::D, Facing::West)),
            ((CubeSide::R, Facing::South), (CubeSide::B, Facing::North)),
            ((CubeSide::R, Facing::West), (CubeSide::U, Facing::East)),
            ((CubeSide::R, Facing::East), (CubeSide::D, Facing::East)),
            ((CubeSide::R, Facing::North), (CubeSide::F, Facing::East)),
            ((CubeSide::B, Facing::South), (CubeSide::L, Facing::West)),
            ((CubeSide::B, Facing::West), (CubeSide::U, Facing::South)),
            ((CubeSide::B, Facing::East), (CubeSide::D, Facing::North)),
            ((CubeSide::B, Facing::North), (CubeSide::R, Facing::South)),
        ];

        for ((side1, facing1), (side2, facing2)) in expected_edges {
            assert_eq!(
                map.cube_layout.edges.get(&(side1, facing1)),
                Some(&(side2, facing2))
            );
            assert_eq!(
                map.cube_layout.edges.get(&(side2, facing2)),
                Some(&(side1, facing1))
            );
        }

        let mut state = State::initial(&map);

        assert_eq!(state.position, Position { x: 2, y: 0 });
        assert_eq!(state.facing, Facing::East);

        for instruction in instructions {
            match instruction {
                Instruction::Move(distance) => {
                    for _ in 0..distance {
                        (state.position, state.facing) =
                            map.advance_position(state.position, state.facing);
                    }
                }
                _ => state.facing = state.facing + instruction,
            }
        }

        assert_eq!(state.position, Position { x: 2, y: 0 });
        assert_eq!(state.facing, Facing::East);
    }
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
            .map(Map::new)
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
}
