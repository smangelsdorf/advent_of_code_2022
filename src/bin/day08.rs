use std::convert::identity;
use std::iter::from_fn;

fn update_visible(acc: u8, tree: &mut (u8, bool)) -> u8 {
    let (h, v) = tree;
    if *h > acc {
        *v = true;
        *h
    } else {
        acc
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut trees: Vec<Vec<(u8, bool)>> = std::io::stdin()
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|l| l.bytes().map(|b| (b, false)).collect())
        .collect();

    // Left to right
    for row in trees.iter_mut() {
        // We're using the byte value so 0 is definitely lower than all.
        row.iter_mut().fold(0, update_visible);
    }

    // Right to left
    for row in trees.iter_mut() {
        row.iter_mut().rev().fold(0, update_visible);
    }

    // Transpose and then do the same again.
    let mut source = trees.into_iter().map(|i| i.into_iter()).collect::<Vec<_>>();
    let mut trees: Vec<Vec<(u8, bool)>> = from_fn(|| {
        let i = source.iter_mut().map(|i| i.next());
        Some(i.filter_map(identity).collect()).filter(|v: &Vec<(u8, bool)>| !v.is_empty())
    })
    .collect();

    for row in trees.iter_mut() {
        row.iter_mut().fold(0, update_visible);
    }

    for row in trees.iter_mut() {
        row.iter_mut().rev().fold(0, update_visible);
    }

    let n: usize = trees
        .into_iter()
        .map(|row| row.into_iter().filter(|(_, visible)| *visible).count())
        .sum();

    println!("{}", n);

    Ok(())
}
