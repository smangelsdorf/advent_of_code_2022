use std::fmt::Debug;
use std::ops::Add;

#[derive(Default)]
struct Points(u64);

impl Add for Points {
    type Output = Points;

    fn add(self, other: Points) -> Self::Output {
        Points(self.0 + other.0)
    }
}

enum Shape {
    Rock,
    Paper,
    Scissors,
}

impl From<&str> for Shape {
    fn from(s: &str) -> Shape {
        use Shape::*;

        match s {
            "A" => Rock,
            "B" => Paper,
            "C" => Scissors,
            _ => panic!("invalid input"),
        }
    }
}

impl From<Shape> for Points {
    fn from(m: Shape) -> Points {
        use Shape::*;

        Points(match m {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        })
    }
}

enum Outcome {
    ElfWin,
    Draw,
    MeWin,
}

impl From<Outcome> for Points {
    fn from(o: Outcome) -> Points {
        use Outcome::*;

        Points(match o {
            ElfWin => 0,
            Draw => 3,
            MeWin => 6,
        })
    }
}

impl From<&str> for Outcome {
    fn from(s: &str) -> Outcome {
        use Outcome::*;

        match s {
            "X" => ElfWin,
            "Y" => Draw,
            "Z" => MeWin,
            _ => panic!("invalid input"),
        }
    }
}

struct Round {
    elf: Shape,
    outcome: Outcome,
}

impl Round {
    fn my_shape(&self) -> Shape {
        use {Outcome::*, Shape::*};

        let Round { elf, outcome } = self;

        match (elf, outcome) {
            (Rock, ElfWin) | (Scissors, Draw) | (Paper, MeWin) => Scissors,
            (Scissors, ElfWin) | (Paper, Draw) | (Rock, MeWin) => Paper,
            (Paper, ElfWin) | (Rock, Draw) | (Scissors, MeWin) => Rock,
        }
    }
}

impl From<Round> for Points {
    fn from(r: Round) -> Points {
        Points::from(r.my_shape()) + Points::from(r.outcome)
    }
}

pub fn run() {
    let score: Points = std::io::stdin()
        .lines()
        .map_while(Result::ok)
        .map(parse_line)
        .map(Points::from)
        .fold(Points::default(), Points::add);
    println!("score: {}", score.0);
}

// We get a `String` here, but this is a way to be generic across `String` and `&str`.
fn parse_line<S>(line: S) -> Round
where
    S: AsRef<str> + Debug,
{
    match line.as_ref().split_once(" ") {
        Some((elf_string, outcome_string)) => Round {
            elf: elf_string.into(),
            outcome: outcome_string.into(),
        },
        None => panic!("invalid line: {:?}", line),
    }
}
