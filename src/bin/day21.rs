use std::collections::HashMap;

use aoc::parser::read_from_stdin_and_parse;

#[derive(Debug)]
struct Unresolved;

#[derive(Debug)]
enum Monkey {
    YellingMonkey {
        value: i64,
    },
    WaitingMonkey {
        refs: (u32, u32),
        operation: Operation,
    },
}

#[derive(Debug)]
enum Operation {
    Add,
    Mul,
    Div,
    Sub,
}

pub fn main() {
    let mut monkeys = read_from_stdin_and_parse(parser::parse_input).unwrap();
    println!("{:?}", monkeys);

    // b"root" as u32 (little endian)
    let root = 0x746f6f72;
    println!("root: {:?}", monkeys.get(&root));

    let mut stack: Vec<u32> = Vec::with_capacity(monkeys.len());
    stack.push(root);

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
                _ => {
                    stack.push(monkey);
                    stack.push(*a);
                    stack.push(*b);
                    println!("stack length: {}", stack.len());
                }
            },
            _ => {}
        }
    }

    // Print root monkey value
    println!(
        "root monkey value: {:?}",
        match monkeys.get(&root) {
            Some(Monkey::YellingMonkey { value }) => value,
            _ => panic!("root monkey is not a yelling monkey"),
        }
    );
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
