use std::ops::RangeInclusive;

use aoc::parser::read_from_stdin_and_parse;
use itertools::Itertools;

#[derive(Default, Debug, Clone, Copy)]
enum Material {
    #[default]
    Air,
    Rock,
    Sand,
}

#[derive(Debug)]
struct Area {
    x_bounds: RangeInclusive<usize>,
    y_bounds: RangeInclusive<usize>,

    grid: Vec<Vec<Material>>,
}

impl Area {
    fn new(x_bounds: RangeInclusive<usize>, y_bounds: RangeInclusive<usize>) -> Area {
        let grid = x_bounds
            .clone()
            .map(|_| y_bounds.clone().map(|_| Material::default()).collect())
            .collect();

        Area {
            x_bounds,
            y_bounds,
            grid,
        }
    }

    fn contains(&self, pos: &Pos) -> bool {
        self.x_bounds.contains(&pos.x) && self.y_bounds.contains(&pos.y)
    }

    fn translate(&self, pos: Pos) -> Option<(usize, usize)> {
        if self.contains(&pos) {
            Some((pos.x - self.x_bounds.start(), pos.y - self.y_bounds.start()))
        } else {
            None
        }
    }

    fn get_at(&self, pos: Pos) -> Option<Material> {
        self.translate(pos)
            .and_then(|(i, j)| self.grid.get(i).and_then(|v| v.get(j)).copied())
    }

    fn set_at(&mut self, pos: Pos, material: Material) {
        self.translate(pos)
            .and_then(|(i, j)| self.grid.get_mut(i).and_then(|v| v.get_mut(j)))
            .map(|v| *v = material);
    }

    fn fall_from(&self, pos: Pos) -> LandingSpace {
        for p in pos.fall_candidates() {
            match self.get_at(p) {
                Some(Material::Air) => return self.fall_from(p),
                None => return LandingSpace::OutOfBounds,
                _ => {}
            }
        }

        LandingSpace::Pos(pos)
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn fall_candidates(self) -> [Pos; 3] {
        let Pos { x, y } = self;

        [
            Pos { x, y: y + 1 },
            Pos { x: x - 1, y: y + 1 },
            Pos { x: x + 1, y: y + 1 },
        ]
    }
}

enum RockLine {
    Horizontal {
        y: usize,
        x_start: usize,
        x_end: usize,
    },

    Vertical {
        x: usize,
        y_start: usize,
        y_end: usize,
    },
}

impl RockLine {
    fn new(start: Pos, end: Pos) -> RockLine {
        if start.x == end.x {
            RockLine::Vertical {
                x: start.x,
                y_start: start.y,
                y_end: end.y,
            }
        } else if start.y == end.y {
            RockLine::Horizontal {
                y: start.y,
                x_start: start.x,
                x_end: end.x,
            }
        } else {
            panic!("bad input data, mismatched points")
        }
    }

    fn positions(&self) -> Box<dyn Iterator<Item = Pos>> {
        match *self {
            RockLine::Horizontal { y, x_start, x_end } => {
                Box::new(range(x_start, x_end).map(move |x| Pos { x, y }))
            }
            RockLine::Vertical { x, y_start, y_end } => {
                Box::new(range(y_start, y_end).map(move |y| Pos { x, y }))
            }
        }
    }
}

#[derive(Debug)]
enum LandingSpace {
    OutOfBounds,
    Pos(Pos),
}

fn range(a: usize, b: usize) -> RangeInclusive<usize> {
    if a < b {
        a..=b
    } else {
        b..=a
    }
}

fn to_bounds<'i, I, F>(iter: I, f: F) -> RangeInclusive<usize>
where
    I: IntoIterator<Item = &'i Vec<Pos>> + 'i,
    F: Fn(Pos) -> usize + Copy,
{
    let (start, end) = iter
        .into_iter()
        .flat_map(move |v| v.iter().copied().map(f))
        .minmax()
        .into_option()
        .unwrap();

    start..=end
}

pub fn main() {
    let data = read_from_stdin_and_parse(parser::parse_input).unwrap();

    let x_bounds = to_bounds(&data, |Pos { x, .. }| x);
    let y_bounds = to_bounds(&data, |Pos { y, .. }| y);
    let y_bounds = 0..=*y_bounds.end();

    let mut area = Area::new(x_bounds, y_bounds);

    let lines = data
        .into_iter()
        .flat_map(|v| {
            v.into_iter()
                .tuple_windows()
                .map(|(a, b)| RockLine::new(a, b))
        })
        .collect::<Vec<RockLine>>();

    for line in lines {
        for pos in line.positions() {
            area.set_at(pos, Material::Rock);
        }
    }

    let origin = Pos { x: 500, y: 0 };

    for i in 0.. {
        let landing = area.fall_from(origin);
        match landing {
            LandingSpace::Pos(pos) => area.set_at(pos, Material::Sand),
            LandingSpace::OutOfBounds => {
                println!("{}", i);
                return;
            }
        }
    }
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space0};
    use nom::combinator::{eof, value};
    use nom::multi::{many0, many1, separated_list1};
    use nom::sequence::{separated_pair, terminated, tuple};
    use nom::{IResult, Parser};

    fn pos(input: &str) -> IResult<&str, Pos> {
        separated_pair(base10_numeric, tag(","), base10_numeric)
            .map(|(x, y)| Pos { x, y })
            .parse(input)
    }

    fn separator(input: &str) -> IResult<&str, ()> {
        value((), tuple((space0, tag("->"), space0))).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Pos>>> {
        terminated(
            separated_list1(many1(line_ending), separated_list1(separator, pos)),
            tuple((many0(line_ending), eof)),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_input() {
            let input = "498,4 -> 498,6 -> 496,6\n\
                         503,4 -> 502,4 -> 502,9 -> 494,9";

            let (_input, data) = parse_input(input).unwrap();

            assert_eq!(
                data,
                vec![
                    vec![
                        Pos { x: 498, y: 4 },
                        Pos { x: 498, y: 6 },
                        Pos { x: 496, y: 6 },
                    ],
                    vec![
                        Pos { x: 503, y: 4 },
                        Pos { x: 502, y: 4 },
                        Pos { x: 502, y: 9 },
                        Pos { x: 494, y: 9 },
                    ]
                ]
            );
        }
    }
}
