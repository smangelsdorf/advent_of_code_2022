use std::collections::BTreeSet;
use std::io::Read;

pub fn main() {
    println!(
        "{}",
        std::io::stdin()
            .bytes()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .windows(4)
            .enumerate()
            .find(|(_, slice)| slice.iter().copied().collect::<BTreeSet<_>>().len() == slice.len())
            .unwrap()
            .0
            + 4
    );
}
