use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq)]
struct Item {
    worry: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct Monkey {
    items: Vec<Item>,
    operation: Operation,
    action: ThrowAction,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MonkeyId(u64);

#[derive(Debug, Eq, PartialEq)]
enum Operand {
    Old,
    Const(u64),
}

#[derive(Debug, Eq, PartialEq)]
enum Operator {
    Add,
    Mul,
}

#[derive(Debug, Eq, PartialEq)]
struct Operation {
    operands: [Operand; 2],
    operator: Operator,
}

#[derive(Debug, Eq, PartialEq)]
enum Test {
    DivisibleBy(i64),
}

#[derive(Debug, Eq, PartialEq)]
struct ThrowAction {
    test: Test,
    true_target: MonkeyId,
    false_target: MonkeyId,
}

pub fn main() {}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space0, space1};
    use nom::combinator::value;
    use nom::multi::{many0, many1, separated_list1};
    use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
    use nom::{IResult, Parser};

    fn field_label(caption: &str) -> impl Parser<&str, (), nom::error::Error<&str>> {
        value((), tuple((space0, tag(caption), tag(":"), space0)))
    }

    fn monkey_id(input: &str) -> IResult<&str, MonkeyId> {
        base10_numeric.map(MonkeyId).parse(input)
    }

    fn monkey_header(input: &str) -> IResult<&str, MonkeyId> {
        delimited(
            tuple((alt((space0, line_ending)), tag("Monkey"), space1)),
            monkey_id,
            tag(":"),
        )
        .parse(input)
    }

    fn items(input: &str) -> IResult<&str, Vec<Item>> {
        preceded(
            field_label("Starting items"),
            separated_list1(
                tuple((tag(","), space1)),
                base10_numeric.map(|worry| Item { worry }),
            ),
        )
        .parse(input)
    }

    fn operator(input: &str) -> IResult<&str, Operator> {
        alt((
            tag("*").map(|_| Operator::Mul),
            tag("+").map(|_| Operator::Add),
        ))
        .parse(input)
    }

    fn operand(input: &str) -> IResult<&str, Operand> {
        alt((
            tag("old").map(|_| Operand::Old),
            base10_numeric.map(|n| Operand::Const(n)),
        ))
        .parse(input)
    }

    fn operation(input: &str) -> IResult<&str, Operation> {
        preceded(
            tuple((
                field_label("Operation"),
                terminated(tag("new"), space0),
                terminated(tag("="), space0),
            )),
            tuple((
                terminated(operand, space0),
                terminated(operator, space0),
                terminated(operand, space0),
            )),
        )
        .map(|(lhs, op, rhs)| Operation {
            operands: [lhs, rhs],
            operator: op,
        })
        .parse(input)
    }

    fn test(input: &str) -> IResult<&str, Test> {
        preceded(
            tuple((field_label("Test"), tag("divisible by"), space1)),
            base10_numeric,
        )
        .map(Test::DivisibleBy)
        .parse(input)
    }

    fn throw_target(input: &str) -> IResult<&str, MonkeyId> {
        preceded(tuple((tag("throw to monkey"), space1)), monkey_id).parse(input)
    }

    fn true_branch(input: &str) -> IResult<&str, MonkeyId> {
        preceded(field_label("If true"), throw_target).parse(input)
    }

    fn false_branch(input: &str) -> IResult<&str, MonkeyId> {
        preceded(field_label("If false"), throw_target).parse(input)
    }

    fn throw_action(input: &str) -> IResult<&str, ThrowAction> {
        tuple((
            terminated(test, line_ending),
            terminated(true_branch, line_ending),
            false_branch,
        ))
        .map(|(test, true_target, false_target)| ThrowAction {
            test,
            true_target,
            false_target,
        })
        .parse(input)
    }

    fn monkey_detail(input: &str) -> IResult<&str, Monkey> {
        tuple((
            terminated(items, line_ending),
            terminated(operation, line_ending),
            terminated(throw_action, line_ending),
        ))
        .map(|(items, operation, action)| Monkey {
            items,
            operation,
            action,
        })
        .parse(input)
    }

    fn monkey(input: &str) -> IResult<&str, (MonkeyId, Monkey)> {
        separated_pair(monkey_header, line_ending, monkey_detail).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, BTreeMap<MonkeyId, Monkey>> {
        preceded(
            many0(line_ending),
            separated_list1(many1(line_ending), monkey),
        )
        .map(|v| v.into_iter().collect())
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_input() {
            let input = r"
                Monkey 0:
                  Starting items: 79, 98
                  Operation: new = old * 19
                  Test: divisible by 23
                    If true: throw to monkey 2
                    If false: throw to monkey 3

                Monkey 1:
                  Starting items: 54, 65, 75, 74
                  Operation: new = old + 6
                  Test: divisible by 19
                    If true: throw to monkey 2
                    If false: throw to monkey 0

                Monkey 2:
                  Starting items: 79, 60, 97
                  Operation: new = old * old
                  Test: divisible by 13
                    If true: throw to monkey 1
                    If false: throw to monkey 3

                Monkey 3:
                  Starting items: 74
                  Operation: new = old + 3
                  Test: divisible by 17
                    If true: throw to monkey 0
                    If false: throw to monkey 1
            ";

            let expected = vec![
                (
                    MonkeyId(0),
                    Monkey {
                        items: vec![Item { worry: 79 }, Item { worry: 98 }],
                        operation: Operation {
                            operands: [Operand::Old, Operand::Const(19)],
                            operator: Operator::Mul,
                        },
                        action: ThrowAction {
                            test: Test::DivisibleBy(23),
                            true_target: MonkeyId(2),
                            false_target: MonkeyId(3),
                        },
                    },
                ),
                (
                    MonkeyId(1),
                    Monkey {
                        items: vec![
                            Item { worry: 54 },
                            Item { worry: 65 },
                            Item { worry: 75 },
                            Item { worry: 74 },
                        ],
                        operation: Operation {
                            operands: [Operand::Old, Operand::Const(6)],
                            operator: Operator::Add,
                        },
                        action: ThrowAction {
                            test: Test::DivisibleBy(19),
                            true_target: MonkeyId(2),
                            false_target: MonkeyId(0),
                        },
                    },
                ),
                (
                    MonkeyId(2),
                    Monkey {
                        items: vec![Item { worry: 79 }, Item { worry: 60 }, Item { worry: 97 }],
                        operation: Operation {
                            operands: [Operand::Old, Operand::Old],
                            operator: Operator::Mul,
                        },
                        action: ThrowAction {
                            test: Test::DivisibleBy(13),
                            true_target: MonkeyId(1),
                            false_target: MonkeyId(3),
                        },
                    },
                ),
                (
                    MonkeyId(3),
                    Monkey {
                        items: vec![Item { worry: 74 }],
                        operation: Operation {
                            operands: [Operand::Old, Operand::Const(3)],
                            operator: Operator::Add,
                        },
                        action: ThrowAction {
                            test: Test::DivisibleBy(17),
                            true_target: MonkeyId(0),
                            false_target: MonkeyId(1),
                        },
                    },
                ),
            ];

            let (_input, monkeys) = parse_input(input).unwrap();
            assert_eq!(monkeys.into_iter().collect::<Vec<_>>(), expected);
        }
    }
}
