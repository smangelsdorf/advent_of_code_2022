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
    // We need the top 3 values, but leave space for an extra value to be sorted in with them.
    maxes: [u64; 4],
    acc: u64,
}

impl State {
    fn add(self, n: u64) -> State {
        let State { maxes, acc } = self;
        State {
            maxes,
            acc: acc + n,
        }
    }

    fn next(self) -> State {
        let State { mut maxes, acc } = self;

        // Always in ascending order. Replace the lowest value and sort again.
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

    // Sneaky .next() to roll over the last accumulator.
    let State { maxes, .. } = r.expect("Processing input").next();

    // Discard the lowest of the 4, it was just left over from the last iteration.
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
