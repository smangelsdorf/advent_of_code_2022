use std::collections::HashSet;

struct Rucksack {
    first: HashSet<u8>,
    second: HashSet<u8>,
}

impl Rucksack {
    fn common(self) -> Option<u8> {
        self.first.intersection(&self.second).next().copied()
    }
}

impl<S> From<S> for Rucksack
where
    S: AsRef<str>,
{
    fn from(s: S) -> Rucksack {
        let s = s.as_ref();
        let (first, second) = s.split_at(s.len() / 2);

        let first = HashSet::from_iter(first.bytes().into_iter());
        let second = HashSet::from_iter(second.bytes().into_iter());

        Rucksack { first, second }
    }
}

fn priority(b: u8) -> u64 {
    let value = if b.is_ascii_lowercase() {
        b - b'a' + 1
    } else if b.is_ascii_uppercase() {
        b - b'A' + 27
    } else {
        panic!("unexpected byte: {}", b);
    };

    u64::from(value)
}

pub fn run() {
    let value: u64 = std::io::stdin()
        .lines()
        .map(Result::unwrap)
        .map(Rucksack::from)
        .filter_map(Rucksack::common)
        .map(priority)
        .sum();
    println!("{}", value);
}
