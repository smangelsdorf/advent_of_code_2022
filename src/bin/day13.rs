use std::cmp::Ordering;

use aoc::parser::read_from_stdin_and_parse;

#[derive(Eq, PartialEq, Debug, Clone)]
struct Packet {
    data: PacketData,
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum PacketData {
    List(Vec<PacketData>),
    Integer(u64),
}

use PacketData::*;

impl PartialOrd for PacketData {
    fn partial_cmp(&self, other: &PacketData) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PacketData {
    fn cmp(&self, other: &PacketData) -> Ordering {
        match (self, other) {
            (List(left), List(right)) => left.cmp(right),
            (Integer(left), Integer(right)) => left.cmp(right),
            (left @ List(_), Integer(right)) => left.cmp(&List(vec![Integer(*right)])),
            (Integer(left), right @ List(_)) => List(vec![Integer(*left)]).cmp(right),
        }
    }
}

pub fn main() {
    let mut data = read_from_stdin_and_parse(parser::parse_input).unwrap();

    let divider1 = Packet {
        data: List(vec![List(vec![Integer(2)])]),
    };
    let divider2 = Packet {
        data: List(vec![List(vec![Integer(6)])]),
    };

    data.push(divider1.clone());
    data.push(divider2.clone());
    data.sort_by(|a, b| a.data.cmp(&b.data));

    let first = data.iter().position(|p| p == &divider1);
    let second = data.iter().position(|p| p == &divider2);

    if let Some((i, j)) = first.zip(second) {
        let n = (i + 1) * (j + 1);
        println!("{}", n);
    }
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::line_ending;
    use nom::combinator::eof;
    use nom::multi::{many0, many1, separated_list0, separated_list1};
    use nom::sequence::{delimited, terminated, tuple};
    use nom::{IResult, Parser};

    fn packet_data(input: &str) -> IResult<&str, PacketData> {
        alt((
            delimited(tag("["), separated_list0(tag(","), packet_data), tag("]")).map(List),
            base10_numeric.map(Integer),
        ))
        .parse(input)
    }

    fn packet(input: &str) -> IResult<&str, Packet> {
        packet_data.map(|data| Packet { data }).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<Packet>> {
        terminated(
            separated_list1(many1(line_ending), packet),
            tuple((many0(line_ending), eof)),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_input() {
            let input = "\
                [[1],[2,3,4]]\n\
                [[1],2]\n\
                \n\
                [1,2,3,4,5]\n\
                [5,4,3,2,[1]]";

            let (_input, pairs) = parse_input(input).unwrap();

            assert_eq!(
                pairs,
                vec![
                    Packet {
                        data: List(vec![
                            List(vec![Integer(1)]),
                            List(vec![Integer(2), Integer(3), Integer(4)])
                        ])
                    },
                    Packet {
                        data: List(vec![List(vec![Integer(1)]), Integer(2)])
                    },
                    Packet {
                        data: List(vec![
                            Integer(1),
                            Integer(2),
                            Integer(3),
                            Integer(4),
                            Integer(5)
                        ])
                    },
                    Packet {
                        data: List(vec![
                            Integer(5),
                            Integer(4),
                            Integer(3),
                            Integer(2),
                            List(vec![Integer(1)])
                        ])
                    },
                ]
            );
        }
    }
}
