use aoc::parser::read_from_stdin_and_parse;

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Addx(i64),
    Noop,
}

#[derive(Copy, Clone, Debug)]
struct State(i64);

impl Default for State {
    fn default() -> State {
        State(1)
    }
}

struct Execution<I>
where
    I: Iterator<Item = Instruction>,
{
    state: State,
    instructions: I,
    ticks: usize,
    next_state: State,
}

impl<I> Execution<I>
where
    I: Iterator<Item = Instruction>,
{
    fn new(i: I) -> Execution<I> {
        Execution {
            state: State(1),
            instructions: i,
            ticks: 0,
            next_state: State(1),
        }
    }
}

impl<I> Iterator for Execution<I>
where
    I: Iterator<Item = Instruction>,
{
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ticks > 0 {
            self.ticks -= 1;
            Some(self.state)
        } else {
            self.state = self.next_state;

            match self.instructions.next() {
                Some(Instruction::Noop) => {
                    self.ticks = 0;
                    Some(self.state)
                }
                Some(Instruction::Addx(n)) => {
                    self.next_state = State(self.state.0 + n);
                    self.ticks = 1;
                    Some(self.state)
                }
                None => None,
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instructions = read_from_stdin_and_parse(parser::parse_input)?;

    let sum: i64 = Execution::new(instructions.into_iter())
        .enumerate()
        .skip(19)
        .step_by(40)
        .take(6)
        .map(|(i, State(n))| (i as i64 + 1) * n)
        .sum();

    println!("{}", sum);

    Ok(())
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space1};
    use nom::combinator::eof;
    use nom::multi::{many0, separated_list1};
    use nom::sequence::{preceded, terminated, tuple};
    use nom::{IResult, Parser};

    fn addx_instruction(input: &str) -> IResult<&str, Instruction> {
        preceded(tuple((tag("addx"), space1)), base10_numeric)
            .map(|n| Instruction::Addx(n))
            .parse(input)
    }

    fn noop_instruction(input: &str) -> IResult<&str, Instruction> {
        tag("noop").map(|_| Instruction::Noop).parse(input)
    }

    fn instruction(input: &str) -> IResult<&str, Instruction> {
        alt((addx_instruction, noop_instruction)).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<Instruction>> {
        terminated(
            separated_list1(line_ending, instruction),
            tuple((many0(line_ending), eof)),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        aoc::parser::base10_numeric::<i64>("-5").unwrap();
        aoc::parser::base10_numeric::<i64>("3").unwrap();
        let input = "\
            noop\n\
            addx 3\n\
            addx -5\n\
        ";

        let (_, instructions) = parser::parse_input(input).unwrap();

        let execution = Execution::new(instructions.into_iter());
        let steps = execution.map(|State(n)| n).collect::<Vec<i64>>();

        assert_eq!(steps, vec![1, 1, 1, 4, 4]);
    }
}
