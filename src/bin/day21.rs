use std::collections::HashMap;

use aoc::parser::read_from_stdin_and_parse;

#[derive(Debug)]
struct Unresolved;

#[derive(Debug)]
enum Monkey {
    Human,
    YellingMonkey {
        value: i64,
    },
    WaitingMonkey {
        refs: (u32, u32),
        operation: Operation,
    },
    EquationMonkey {
        equation: Equation,
    },
}

#[derive(Debug, Clone, Copy)]
struct Equation {
    refs: (u32, u32),
    operation: Operation,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add,
    Mul,
    Div,
    Sub,
}

pub fn main() {
    let mut monkeys = read_from_stdin_and_parse(parser::parse_input).unwrap();

    // b"root" as u32 (little endian)
    let root = 0x746f6f72;
    let humn = 0x6e6d7568;

    let mut stack: Vec<u32> = Vec::with_capacity(monkeys.len());

    let root_monkey = monkeys.remove(&root).unwrap();
    let (root_a, root_b) = match root_monkey {
        Monkey::WaitingMonkey { refs: (a, b), .. } => {
            stack.push(a);
            stack.push(b);

            (a, b)
        }
        _ => panic!("root monkey is not a waiting monkey"),
    };

    monkeys.insert(humn, Monkey::Human);

    while let Some(monkey) = stack.pop() {
        match monkeys.get(&monkey) {
            Some(Monkey::WaitingMonkey {
                refs: (a, b),
                operation,
            }) => match Option::zip(monkeys.get(a), monkeys.get(b)) {
                Some((Monkey::YellingMonkey { value: a }, Monkey::YellingMonkey { value: b })) => {
                    let value = match operation {
                        Operation::Add => a + b,
                        Operation::Mul => a * b,
                        Operation::Div => a / b,
                        Operation::Sub => a - b,
                    };
                    monkeys.insert(monkey, Monkey::YellingMonkey { value });
                }
                Some((Monkey::YellingMonkey { .. }, Monkey::EquationMonkey { .. }))
                | Some((Monkey::EquationMonkey { .. }, Monkey::YellingMonkey { .. }))
                | Some((Monkey::Human { .. }, Monkey::YellingMonkey { .. }))
                | Some((Monkey::YellingMonkey { .. }, Monkey::Human { .. })) => {
                    monkeys.insert(
                        monkey,
                        Monkey::EquationMonkey {
                            equation: Equation {
                                refs: (*a, *b),
                                operation: *operation,
                            },
                        },
                    );
                }
                _ => {
                    stack.push(monkey);
                    stack.push(*a);
                    stack.push(*b);
                }
            },
            _ => {}
        }
    }

    let n = match Option::zip(monkeys.get(&root_a), monkeys.get(&root_b)) {
        Some((&Monkey::YellingMonkey { value }, &Monkey::EquationMonkey { equation }))
        | Some((&Monkey::EquationMonkey { equation }, &Monkey::YellingMonkey { value })) => {
            solve(monkeys, value, equation)
        }
        _ => panic!("expected an equation monkey and a yelling monkey"),
    };

    println!("{}", n);
}

fn solve(monkeys: HashMap<u32, Monkey>, target: i64, equation: Equation) -> i64 {
    let Equation {
        refs: (a, b),
        operation,
    } = equation;

    match Option::zip(monkeys.get(&a), monkeys.get(&b)) {
        Some((&Monkey::YellingMonkey { value }, &Monkey::EquationMonkey { equation })) => {
            match operation {
                Operation::Add => solve(monkeys, target - value, equation),
                Operation::Sub => solve(monkeys, value - target, equation),
                Operation::Mul => solve(monkeys, target / value, equation),
                Operation::Div => solve(monkeys, value / target, equation),
            }
        }
        Some((&Monkey::EquationMonkey { equation }, &Monkey::YellingMonkey { value })) => {
            match operation {
                Operation::Add => solve(monkeys, target - value, equation),
                Operation::Sub => solve(monkeys, target + value, equation),
                Operation::Mul => solve(monkeys, target / value, equation),
                Operation::Div => solve(monkeys, target * value, equation),
            }
        }
        Some((&Monkey::YellingMonkey { value }, Monkey::Human)) => match operation {
            Operation::Add => target - value,
            Operation::Sub => value - target,
            Operation::Mul => target / value,
            Operation::Div => value / target,
        },
        Some((Monkey::Human, &Monkey::YellingMonkey { value })) => match operation {
            Operation::Add => target - value,
            Operation::Sub => target + value,
            Operation::Mul => target / value,
            Operation::Div => target * value,
        },
        _ => panic!("expected an equation monkey and a yelling monkey"),
    }
}

mod parser {
    use super::*;
    use aoc::parser::base10_numeric;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_while},
        character::complete::{line_ending, space1},
        combinator::map,
        multi::separated_list1,
        sequence::{preceded, terminated, tuple},
        IResult, Parser,
    };

    fn operator(input: &str) -> IResult<&str, Operation> {
        alt((
            map(tag("+"), |_| Operation::Add),
            map(tag("*"), |_| Operation::Mul),
            map(tag("/"), |_| Operation::Div),
            map(tag("-"), |_| Operation::Sub),
        ))
        .parse(input)
    }

    fn name(input: &str) -> IResult<&str, u32> {
        // Parse a 4-byte alphanumeric name and return it as a u32
        map(
            take_while(|c: char| c.is_ascii_alphanumeric()),
            |s: &str| {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(s.as_bytes());
                u32::from_le_bytes(bytes)
            },
        )
        .parse(input)
    }

    fn operation(input: &str) -> IResult<&str, (u32, Operation, u32)> {
        tuple((terminated(name, space1), operator, preceded(space1, name))).parse(input)
    }

    fn monkey(input: &str) -> IResult<&str, (u32, Monkey)> {
        tuple((
            terminated(name, tag(": ")),
            alt((
                base10_numeric.map(|value| Monkey::YellingMonkey { value }),
                operation.map(|(a, op, b)| Monkey::WaitingMonkey {
                    refs: (a, b),
                    operation: op,
                }),
            )),
        ))
        .parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, HashMap<u32, Monkey>> {
        separated_list1(line_ending, monkey)
            .map(|monkeys| monkeys.into_iter().collect())
            .parse(input)
    }
}
