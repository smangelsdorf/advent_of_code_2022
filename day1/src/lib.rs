enum Input {
    Number(u64),
    Blank,
}

#[derive(Debug)]
enum Error {
    IoError(std::io::Error),
    InvalidInput,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

#[derive(Default)]
struct State {
    maxes: [u64; 4],
    acc: u64,
}

impl State {
    fn add(self, n: u64) -> State {
        let State { acc, .. } = self;
        State {
            acc: acc + n,
            ..self
        }
    }

    fn next(self) -> State {
        let State { mut maxes, acc } = self;
        maxes[0] = acc;
        maxes.sort();

        State { maxes, acc: 0 }
    }
}

pub fn run() {
    let r: Result<_, Error> = std::io::stdin()
        .lines()
        .try_fold(State::default(), |state, item| {
            let s = item?;
            match parse(&s)? {
                Input::Number(n) => Ok(state.add(n)),
                Input::Blank => Ok(state.next()),
            }
        });

    let State { maxes, .. } = r.expect("Processing input").next();
    let [_, a, b, c] = maxes;
    let max = a + b + c;
    println!("{}", max);
}

fn parse(line: &str) -> Result<Input, Error> {
    if line == "" {
        Ok(Input::Blank)
    } else if let Ok(n) = u64::from_str_radix(line, 10) {
        Ok(Input::Number(n))
    } else {
        Err(Error::InvalidInput)
    }
}
