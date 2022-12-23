use std::collections::BTreeSet;

use aoc::parser::{base10_numeric, read_from_stdin_and_parse};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    multi::separated_list1,
    sequence::{terminated, tuple},
    IResult, Parser,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Block {
    Cube,
    Air,
    Steam,
}

#[derive(Debug)]
struct Grid {
    blocks: Vec<Vec<Vec<Block>>>,
}

impl Grid {
    fn new(max_x: usize, max_y: usize, max_z: usize) -> Self {
        Self {
            blocks: vec![vec![vec![Block::Air; max_z + 1]; max_y + 1]; max_x + 1],
        }
    }

    fn from_cube_list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Pos> + Clone,
    {
        let (max_x, max_y, max_z) = iter
            .clone()
            .into_iter()
            .fold((0, 0, 0), |(max_x, max_y, max_z), Pos { x, y, z }| {
                (max_x.max(x), max_y.max(y), max_z.max(z))
            });

        let mut grid = Grid::new(max_x, max_y, max_z);

        for Pos { x, y, z } in iter {
            grid.blocks[x][y][z] = Block::Cube;
        }

        grid
    }

    fn size(&self) -> (usize, usize, usize) {
        (
            self.blocks.len(),
            self.blocks[0].len(),
            self.blocks[0][0].len(),
        )
    }

    fn cubes(&'_ self) -> impl Iterator<Item = Pos> + '_ {
        let (max_x, max_y, max_z) = self.size();

        (0..max_x)
            .cartesian_product((0..max_y).cartesian_product(0..max_z))
            .map(|(x, (y, z))| Pos { x, y, z })
            .filter(|Pos { x, y, z }| self.blocks[*x][*y][*z] == Block::Cube)
    }

    fn neighbour_positions(
        &self,
        x: usize,
        y: usize,
        z: usize,
    ) -> impl Iterator<Item = (usize, usize, usize)> {
        let (max_x, max_y, max_z) = self.size();

        [
            (x.saturating_sub(1), y, z),
            (x + 1, y, z),
            (x, y.saturating_sub(1), z),
            (x, y + 1, z),
            (x, y, z.saturating_sub(1)),
            (x, y, z + 1),
        ]
        .into_iter()
        .filter(move |(x, y, z)| {
            (0..max_x).contains(x) && (0..max_y).contains(y) && (0..max_z).contains(z)
        })
    }

    fn flood_steam(&mut self) {
        let mut seen = BTreeSet::new();
        let (max_x, max_y, max_z) = self.size();

        let iter = (0..max_x)
            .zip((0..max_y).zip(0..max_z))
            .map(|(x, (y, z))| (x, y, z));

        let mut current: Vec<_> = iter
            .clone()
            .filter(|&(x, y, z)| {
                (x == 0 || x == max_x - 1 || y == 0 || y == max_y - 1 || z == 0 || z == max_z - 1)
                    && self.blocks[x][y][z] == Block::Air
            })
            .collect();
        seen.extend(current.clone());

        while let Some((x, y, z)) = current.pop() {
            for (x, y, z) in self.neighbour_positions(x, y, z) {
                if self.blocks[x][y][z] == Block::Air {
                    self.blocks[x][y][z] = Block::Steam;
                    seen.insert((x, y, z));
                    current.push((x, y, z));
                }
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cubes: Vec<Pos> = read_from_stdin_and_parse(parse_input)?;
    let grid = Grid::from_cube_list(cubes.iter().copied());
    let (max_x, max_y, max_z) = grid.size();

    let n = grid
        .cubes()
        .flat_map(|cube| {
            // Air neighbours
            let mut airs = grid
                .neighbour_positions(cube.x, cube.y, cube.z)
                .filter(|&(x, y, z)| grid.blocks[x][y][z] == Block::Air)
                .collect::<Vec<_>>();

            if cube.x == 0 || cube.x == max_x - 1 {
                airs.push((cube.x, cube.y, cube.z));
            }

            if cube.y == 0 || cube.y == max_y - 1 {
                airs.push((cube.x, cube.y, cube.z));
            }

            if cube.z == 0 || cube.z == max_z - 1 {
                airs.push((cube.x, cube.y, cube.z));
            }

            airs
        })
        .count();

    println!("{}", n);

    Ok(())
}

fn parse_input(input: &str) -> IResult<&str, Vec<Pos>> {
    separated_list1(
        line_ending,
        tuple((
            terminated(base10_numeric, tag(",")),
            terminated(base10_numeric, tag(",")),
            base10_numeric,
        ))
        .map(|(x, y, z)| Pos { x, y, z }),
    )
    .parse(input)
}
