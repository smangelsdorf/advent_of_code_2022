use std::collections::BTreeMap;
use std::convert::identity;
use std::io::BufRead;
use std::iter::from_fn;

// Accumulate the most recent tree of each size in a map, use to update the scores.
fn update_visible(
    mut acc: BTreeMap<u8, usize>,
    tree: (usize, &mut (u8, usize)),
) -> BTreeMap<u8, usize> {
    let (index, (h, v)) = tree;

    // The max position of a tree the same size or greater is the closest in this direction.
    let pos = acc.range(*h..u8::MAX).map(|(_k, v)| *v).max().unwrap_or(0);

    // Trees at the edge get zeroed out here.
    *v *= index - pos;

    acc.insert(*h, index);

    acc
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut trees: Vec<Vec<(u8, usize)>> = std::io::stdin()
        .lock()
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|l| l.bytes().map(|b| (b, 1)).collect())
        .collect();

    // Left to right
    for row in trees.iter_mut() {
        row.iter_mut()
            .enumerate()
            .fold(BTreeMap::default(), update_visible);
    }

    // Right to left
    for row in trees.iter_mut() {
        row.iter_mut()
            .rev()
            .enumerate()
            .fold(BTreeMap::default(), update_visible);
    }

    // Transpose and then do the same again.
    let mut source = trees.into_iter().map(|i| i.into_iter()).collect::<Vec<_>>();
    let mut trees: Vec<Vec<(u8, usize)>> = from_fn(|| {
        let i = source.iter_mut().map(|i| i.next());
        Some(i.filter_map(identity).collect::<Vec<_>>()).filter(|v| !v.is_empty())
    })
    .collect();

    for row in trees.iter_mut() {
        row.iter_mut()
            .enumerate()
            .fold(BTreeMap::default(), update_visible);
    }

    for row in trees.iter_mut() {
        row.iter_mut()
            .rev()
            .enumerate()
            .fold(BTreeMap::default(), update_visible);
    }

    let n: usize = trees
        .into_iter()
        .filter_map(|row| row.into_iter().map(|(_, score)| score).max())
        .max()
        .unwrap();

    println!("{}", n);

    Ok(())
}
