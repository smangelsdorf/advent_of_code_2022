use std::collections::BTreeMap;

use aoc::parser::read_from_stdin_and_parse;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Item {
    worry: u64,
}

impl Item {
    fn relieve(self) -> Item {
        Item {
            worry: self.worry / 3,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Monkey {
    items: Vec<Item>,
    operation: Operation,
    action: ThrowAction,
    inspections: u64,
}

impl Monkey {
    // I like the symmetry.
    #[allow(dead_code)]
    fn get(monkeys: &BTreeMap<MonkeyId, Monkey>, id: MonkeyId) -> &Monkey {
        monkeys.get(&id).expect("monkey id should exist")
    }

    fn get_mut(monkeys: &mut BTreeMap<MonkeyId, Monkey>, id: MonkeyId) -> &mut Monkey {
        monkeys.get_mut(&id).expect("monkey id should exist")
    }

    fn catch(&mut self, item: Item) {
        self.items.push(item);
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct MonkeyId(u64);

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Operand {
    Old,
    Const(u64),
}

impl Operand {
    fn of(self, item: Item) -> u64 {
        match self {
            Operand::Old => item.worry,
            Operand::Const(n) => n,
        }
    }
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

impl Operation {
    fn apply(&self, item: Item) -> Item {
        let [lhs, rhs] = self.operands;
        let lhs = lhs.of(item);
        let rhs = rhs.of(item);

        match self.operator {
            Operator::Add => Item { worry: lhs + rhs },
            Operator::Mul => Item { worry: lhs * rhs },
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Test {
    DivisibleBy(u64),
}

impl Test {
    fn matches(&self, item: Item) -> bool {
        match self {
            Test::DivisibleBy(n) => item.worry % n == 0,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct ThrowAction {
    test: Test,
    true_target: MonkeyId,
    false_target: MonkeyId,
}

impl ThrowAction {
    fn identify_target(&self, item: Item) -> MonkeyId {
        if self.test.matches(item) {
            self.true_target
        } else {
            self.false_target
        }
    }
}

#[derive(Debug)]
struct Plan {
    items: Vec<Item>,
    action: ThrowAction,
}

impl Plan {
    fn build(monkey: &mut Monkey) -> Plan {
        let Monkey {
            items,
            operation,
            action,
            inspections,
        } = monkey;

        *inspections += items.len() as u64;

        let items = items
            .drain(..)
            .map(|i| operation.apply(i).relieve())
            .collect();

        Plan {
            items,
            action: *action,
        }
    }

    fn execute(self, monkeys: &mut BTreeMap<MonkeyId, Monkey>) {
        for item in self.items.into_iter() {
            let id = self.action.identify_target(item);
            Monkey::get_mut(monkeys, id).catch(item);
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut monkeys = read_from_stdin_and_parse(parser::parse_input)?;
    let ids: Vec<MonkeyId> = monkeys.keys().copied().collect();

    for _i in 0..20 {
        for id in ids.iter() {
            let plan = Plan::build(Monkey::get_mut(&mut monkeys, *id));
            plan.execute(&mut monkeys);
        }
    }

    let mut values: Vec<u64> = monkeys.values().map(|monkey| monkey.inspections).collect();
    values.sort_by(|a, b| b.cmp(a));
    println!("{}", values.iter().take(2).product::<u64>());

    Ok(())
}

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
            throw_action,
        ))
        .map(|(items, operation, action)| Monkey {
            items,
            operation,
            action,
            inspections: 0,
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
                        inspections: 0,
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
                        inspections: 0,
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
                        inspections: 0,
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
                        inspections: 0,
                    },
                ),
            ];

            let (_input, monkeys) = parse_input(input).unwrap();
            assert_eq!(monkeys.into_iter().collect::<Vec<_>>(), expected);
        }
    }
}
