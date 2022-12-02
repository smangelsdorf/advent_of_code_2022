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
            "X" => Rock,
            "Y" => Paper,
            "Z" => Scissors,
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

struct Round {
    elf: Shape,
    me: Shape,
}

impl Round {
    fn outcome(&self) -> Outcome {
        use {Outcome::*, Shape::*};

        let Round { elf, me } = self;

        match (elf, me) {
            (Rock, Scissors) | (Scissors, Paper) | (Paper, Rock) => ElfWin,
            (Rock, Rock) | (Scissors, Scissors) | (Paper, Paper) => Draw,
            (Rock, Paper) | (Scissors, Rock) | (Paper, Scissors) => MeWin,
        }
    }
}

impl From<Round> for Points {
    fn from(r: Round) -> Points {
        Points::from(r.outcome()) + Points::from(r.me)
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

fn parse_line<S>(line: S) -> Round
where
    S: AsRef<str> + Debug,
{
    match line.as_ref().split_once(" ") {
        Some((elf_string, me_string)) => Round {
            elf: elf_string.into(),
            me: me_string.into(),
        },
        None => panic!("invalid line: {:?}", line),
    }
}
