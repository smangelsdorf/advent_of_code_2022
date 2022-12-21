use std::io::Read;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
}

impl std::ops::Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

impl From<u8> for Direction {
    fn from(c: u8) -> Direction {
        match c {
            b'<' => Direction::Left,
            b'>' => Direction::Right,
            _ => panic!("Invalid direction input"),
        }
    }
}

impl Direction {
    fn apply(self, pos: Pos) -> Pos {
        match self {
            Direction::Left => Pos {
                x: pos.x.saturating_sub(1),
                y: pos.y,
            },
            Direction::Right => Pos {
                x: pos.x + 1,
                y: pos.y,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Block {
    Rock,
    Empty,
}

#[derive(Debug)]
struct Cave<const WIDTH: usize> {
    blocks: Vec<[Block; WIDTH]>,
    top: Option<usize>,
}

impl<const WIDTH: usize> Cave<WIDTH> {
    const INCREMENT: [[Block; WIDTH]; 32] = [[Block::Empty; WIDTH]; 32];

    fn new() -> Self {
        let blocks = Vec::from(Cave::<WIDTH>::INCREMENT);

        Self { blocks, top: None }
    }

    fn rock_start_position(&self) -> Pos {
        Pos {
            x: 2,
            y: self.top.map(|y| y + 4).unwrap_or(3),
        }
    }

    fn get(&self, pos: Pos) -> Option<Block> {
        self.blocks.get(pos.y).and_then(|v| v.get(pos.x)).copied()
    }

    fn put_rock(&mut self, pos: Pos) {
        self.blocks[pos.y][pos.x] = Block::Rock;

        match self.top {
            Some(y) if y > pos.y => {}
            _ => {
                self.top = Some(pos.y);

                if pos.y + 10 > self.blocks.len() {
                    self.blocks.extend_from_slice(&Cave::<WIDTH>::INCREMENT);
                }
            }
        }
    }

    fn valid(&self, rock: &dyn RockShape, offset: Pos) -> bool {
        rock.positions()
            .all(|pos| self.get(offset + pos) == Some(Block::Empty))
    }
}

trait RockShape {
    fn positions(&self) -> RockPosIter;
}

#[derive(Debug, Default, Clone, Copy)]
struct RockPosIter {
    n: usize,
    positions: [Pos; 5],
}

impl Iterator for RockPosIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        match self.n.checked_sub(1) {
            Some(n) => {
                self.n = n;
                Some(self.positions[n])
            }
            _ => None,
        }
    }
}

const ROCK_SHAPES: [&'static dyn RockShape; 5] = [
    &Rock {
        blocks: [[Block::Rock; 4]; 1],
    },
    &Rock {
        blocks: [
            [Block::Empty, Block::Rock, Block::Empty],
            [Block::Rock, Block::Rock, Block::Rock],
            [Block::Empty, Block::Rock, Block::Empty],
        ],
    },
    &Rock {
        blocks: [
            [Block::Rock, Block::Rock, Block::Rock],
            [Block::Empty, Block::Empty, Block::Rock],
            [Block::Empty, Block::Empty, Block::Rock],
        ],
    },
    &Rock {
        blocks: [[Block::Rock; 1]; 4],
    },
    &Rock {
        blocks: [[Block::Rock; 2]; 2],
    },
];

#[derive(Debug)]
struct Rock<const WIDTH: usize, const HEIGHT: usize> {
    blocks: [[Block; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> RockShape for Rock<WIDTH, HEIGHT> {
    fn positions(&self) -> RockPosIter {
        let mut out = RockPosIter::default();

        let position_iter = (0..HEIGHT)
            .rev()
            .flat_map(|y| (0..WIDTH).rev().map(move |x| Pos { x, y }))
            .filter(|Pos { x, y }| self.blocks[*y][*x] == Block::Rock)
            .enumerate();

        for (i, pos) in position_iter {
            out.positions[i] = pos;
            out.n = i + 1;
        }

        out
    }
}

pub fn main() {
    let mut jet_pattern_iter = std::io::stdin()
        .bytes()
        .map(|r| r.map(Direction::from))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .cycle();

    let mut rocks_iter = ROCK_SHAPES.into_iter().cycle();

    let mut cave = Cave::<7>::new();

    for _i in 0..2022 {
        let rock = rocks_iter.next().unwrap();
        let mut pos = cave.rock_start_position();

        loop {
            let direction = jet_pattern_iter.next().unwrap();
            let candidate = direction.apply(pos);

            if cave.valid(rock, candidate) {
                pos = candidate;
            }

            let candidate = pos.y.checked_sub(1).map(|y| Pos { x: pos.x, y });

            match candidate {
                Some(candidate) if cave.valid(rock, candidate) => {
                    pos = candidate;
                }
                _ => {
                    for rock_pos in rock.positions() {
                        cave.put_rock(pos + rock_pos);
                    }

                    break;
                }
            }
        }
    }

    println!("{}", cave.top.unwrap() + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rock_pos_iter() {
        let rock = Rock {
            blocks: [[Block::Rock; 4]; 1],
        };

        assert_eq!(
            rock.positions().collect::<Vec<_>>(),
            vec![
                Pos { x: 0, y: 0 },
                Pos { x: 1, y: 0 },
                Pos { x: 2, y: 0 },
                Pos { x: 3, y: 0 }
            ]
        );

        let rock = Rock {
            blocks: [[Block::Empty, Block::Rock], [Block::Rock, Block::Rock]],
        };

        assert_eq!(
            rock.positions().collect::<Vec<_>>(),
            vec![Pos { x: 1, y: 0 }, Pos { x: 0, y: 1 }, Pos { x: 1, y: 1 },]
        );
    }
}
