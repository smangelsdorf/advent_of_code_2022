use aoc::parser::{base10_numeric, read_from_stdin_and_parse};
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    multi::separated_list1,
    sequence::{terminated, tuple},
    IResult, Parser,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cubes: Vec<(usize, usize, usize)> = read_from_stdin_and_parse(parse_input)?;

    let (max_x, max_y, max_z) = cubes
        .iter()
        .fold((0, 0, 0), |(max_x, max_y, max_z), (x, y, z)| {
            (max_x.max(*x), max_y.max(*y), max_z.max(*z))
        });

    let mut grid = vec![vec![vec![false; max_z + 1]; max_y + 1]; max_x + 1];
    for (x, y, z) in cubes {
        grid[x][y][z] = true;
    }

    let mut n = 0;

    for (x0, y0, z0) in grid
        .iter()
        .enumerate()
        .flat_map(|(x, yz)| {
            yz.iter()
                .enumerate()
                .flat_map(move |(y, z)| z.iter().enumerate().map(move |(z, _)| (x, y, z)))
        })
        .filter(|(x, y, z)| grid[*x][*y][*z])
    {
        // Count surrounding cubes
        let count = [
            (x0.saturating_sub(1), y0, z0),
            (x0 + 1, y0, z0),
            (x0, y0.saturating_sub(1), z0),
            (x0, y0 + 1, z0),
            (x0, y0, z0.saturating_sub(1)),
            (x0, y0, z0 + 1),
        ]
        .into_iter()
        .filter(|(x, y, z)| *x != x0 || *y != y0 || *z != z0)
        .filter(|(x, y, z)| *x <= max_x && *y <= max_y && *z <= max_z)
        .filter(|(x, y, z)| grid[*x][*y][*z])
        .count();

        n += 6 - count;
    }

    println!("{}", n);

    Ok(())
}

fn parse_input(input: &str) -> IResult<&str, Vec<(usize, usize, usize)>> {
    separated_list1(
        line_ending,
        tuple((
            terminated(base10_numeric, tag(",")),
            terminated(base10_numeric, tag(",")),
            base10_numeric,
        )),
    )
    .parse(input)
}
