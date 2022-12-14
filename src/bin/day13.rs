use std::cmp::Ordering;

use aoc::parser::read_from_stdin_and_parse;

#[derive(Eq, PartialEq, Debug)]
struct Packet {
    data: PacketData,
}

#[derive(Eq, PartialEq, Debug)]
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
    let data = read_from_stdin_and_parse(parser::parse_input).unwrap();
    let n = data
        .into_iter()
        .enumerate()
        .filter(|(i, (a, b))| a.data <= b.data)
        .map(|(i, (a, b))| i + 1)
        .sum::<usize>();

    println!("{}", n);
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{line_ending, space0, space1};
    use nom::combinator::eof;
    use nom::multi::{many0, many1, separated_list0, separated_list1};
    use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
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

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<(Packet, Packet)>> {
        terminated(
            separated_list1(
                tuple((line_ending, many1(line_ending))),
                separated_pair(packet, line_ending, packet),
            ),
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
                    (
                        Packet {
                            data: List(vec![
                                List(vec![Integer(1)]),
                                List(vec![Integer(2), Integer(3), Integer(4)])
                            ])
                        },
                        Packet {
                            data: List(vec![List(vec![Integer(1)]), Integer(2)])
                        },
                    ),
                    (
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
                    )
                ]
            );
        }
    }
}
