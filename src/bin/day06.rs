use std::collections::BTreeSet;
use std::io::Read;

const N: usize = 14;

pub fn main() {
    println!(
        "{}",
        std::io::stdin()
            .bytes()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .windows(N)
            .enumerate()
            .find(|(_, slice)| slice.iter().copied().collect::<BTreeSet<_>>().len() == slice.len())
            .unwrap()
            .0
            + N
    );
}
