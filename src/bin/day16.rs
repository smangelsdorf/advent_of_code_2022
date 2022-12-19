use id_arena::Id;
use im::HashSet as ImHashSet;

use aoc::{parser::read_from_stdin_and_parse, MoreIter};
use graph::{ValveGraph, ValveNode};

#[derive(Debug, PartialEq, Clone)]
struct Valve {
    name: String,
    flow_rate: u64,
}

const START_VALVE: &'static str = "AA";
const TIME_LIMIT: u64 = 30;

#[derive(Debug)]
struct State<'a> {
    node: &'a ValveNode,
    flow_rate: u64,
    released: u64,
    time_spent: u64,
    visited: ImHashSet<Id<ValveNode>>,
}

impl<'a> State<'a> {
    fn new(node: &'a ValveNode) -> State<'a> {
        State {
            node,
            flow_rate: 0,
            released: 0,
            time_spent: 0,
            visited: ImHashSet::unit(node.id),
        }
    }

    fn is_valid(&self) -> bool {
        self.time_spent < TIME_LIMIT
    }

    fn is_complete(&self) -> bool {
        self.time_spent == TIME_LIMIT
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = read_from_stdin_and_parse(parser::parse_input)?;

    let mut terminal_states = Vec::with_capacity(1024);
    let mut current = vec![State::new(graph.start())];
    while !current.is_empty() {
        let (next, invalid) = current
            .into_iter()
            .flat_map(|state| {
                let State {
                    node,
                    flow_rate,
                    released,
                    time_spent,
                    visited,
                } = state;

                let graph = &graph;

                // The result of idling here until the time expires.
                let remaining_time = TIME_LIMIT - time_spent;
                let done = State {
                    node,
                    flow_rate,
                    released: released + remaining_time * flow_rate,
                    time_spent: TIME_LIMIT,
                    visited: visited.clone(),
                };

                node.connections
                    .iter()
                    .filter_map(move |conn| {
                        let node = graph.get(conn.target);
                        if visited.contains(&node.id) {
                            None
                        } else {
                            Some(State {
                                node,
                                flow_rate: flow_rate + node.valve.flow_rate,
                                released: released + conn.cost * flow_rate + flow_rate,
                                time_spent: time_spent + conn.cost + 1,
                                visited: visited.update(node.id),
                            })
                        }
                    })
                    .once_if_empty(done)
            })
            .partition::<Vec<_>, _>(State::is_valid);

        terminal_states.extend(invalid.into_iter().filter(State::is_complete));

        current = next;
    }

    let n = terminal_states
        .into_iter()
        .max_by_key(|s| s.released)
        .unwrap();

    println!("{:?}", n);

    Ok(())
}

mod graph {
    use super::*;

    use std::collections::{HashMap, HashSet};

    use id_arena::{Arena, Id};

    #[derive(Debug)]
    pub(super) struct ValveNode {
        pub id: Id<ValveNode>,
        pub valve: Valve,
        pub connections: Vec<ValveNodeConnection>,
    }

    #[derive(Debug)]
    pub(super) struct ValveNodeConnection {
        pub target: Id<ValveNode>,
        pub cost: u64,
    }

    pub(super) struct ValveGraph {
        arena: Arena<ValveNode>,
        start: Id<ValveNode>,
    }

    impl ValveGraph {
        pub(super) fn build(v: Vec<(Valve, Vec<&str>)>) -> ValveGraph {
            v.iter()
                .fold(ValveGraphBuilder::default(), ValveGraphBuilder::accumulate)
                .finish()
        }

        pub fn start(&self) -> &ValveNode {
            self.get(self.start)
        }

        pub fn get(&self, id: Id<ValveNode>) -> &ValveNode {
            self.arena.get(id).expect("valid id")
        }
    }

    #[derive(Default)]
    struct ValveGraphBuilder<'a> {
        arena: Arena<ValveNode>,
        connections: HashMap<&'a str, Vec<&'a str>>,
        ids: HashMap<&'a str, Id<ValveNode>>,
    }

    impl<'a> ValveGraphBuilder<'a> {
        fn accumulate(
            mut self,
            (valve, connections): &'a (Valve, Vec<&'a str>),
        ) -> ValveGraphBuilder<'a> {
            self.connections.insert(&valve.name, connections.clone());

            if valve.name == START_VALVE || valve.flow_rate > 0 {
                let id = self.arena.alloc_with_id(|id| ValveNode {
                    id,
                    valve: valve.clone(),
                    connections: Default::default(),
                });

                self.ids.insert(&valve.name, id);
            }

            self
        }

        fn finish(self) -> ValveGraph {
            let ValveGraphBuilder {
                mut arena,
                connections,
                ids,
            } = self;

            for (&name, &id) in &ids {
                let node = arena.get_mut(id).expect("valid id");
                node.connections = walk_connections(name, &connections, &ids)
            }

            let start = *ids.get(START_VALVE).expect("start valve");

            ValveGraph { arena, start }
        }
    }

    fn walk_connections<'a>(
        start: &'a str,
        connections: &HashMap<&'a str, Vec<&'a str>>,
        ids: &HashMap<&'a str, Id<ValveNode>>,
    ) -> Vec<ValveNodeConnection> {
        let mut seen: HashSet<&'a str> = HashSet::default();

        seen.insert(start);

        let mut current = HashSet::from([start]);
        let mut out = vec![];

        for cost in 1.. {
            current = current
                .into_iter()
                .flat_map(|n| connections.get(n).expect("connection is present"))
                .copied()
                .filter(|n| !seen.contains(n))
                .collect();

            seen.extend(current.iter());

            for n in &current {
                if let Some(&target) = ids.get(n) {
                    out.push(ValveNodeConnection { target, cost });
                }
            }

            if current.is_empty() {
                break;
            }
        }

        out
    }
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, line_ending},
        combinator::eof,
        multi::separated_list1,
        sequence::{preceded, terminated, tuple},
        IResult, Parser,
    };

    fn valve_fields(input: &str) -> IResult<&str, (Valve, Vec<&str>)> {
        tuple((
            preceded(tag("Valve "), alpha1),
            preceded(tag(" has flow rate="), base10_numeric),
            preceded(
                alt((
                    tag("; tunnels lead to valves "),
                    tag("; tunnel leads to valve "),
                )),
                separated_list1(tag(", "), alpha1),
            ),
        ))
        .map(|(name, flow_rate, tunnels)| {
            (
                Valve {
                    name: name.to_owned(),
                    flow_rate,
                },
                tunnels,
            )
        })
        .parse(input)
    }

    fn valves(input: &str) -> IResult<&str, Vec<(Valve, Vec<&str>)>> {
        separated_list1(line_ending, valve_fields).parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, ValveGraph> {
        terminated(valves, eof).map(ValveGraph::build).parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_valves() {
            let input = "Valve AA has flow rate=0; tunnels lead to valves DD, II\n\
                         Valve DD has flow rate=20; tunnels lead to valves AA\n\
                         Valve II has flow rate=0; tunnels lead to valves AA, JJ\n\
                         Valve JJ has flow rate=21; tunnel leads to valve II";

            let result = valves(input);
            assert!(result.is_ok());
            let (remaining_input, entries) = result.unwrap();

            let mut iter = entries.into_iter();

            let (node, connections) = iter.next().expect("AA");
            assert_eq!(node.name, "AA");
            assert_eq!(node.flow_rate, 0);
            assert_eq!(connections, vec!["DD", "II"]);

            let (node, connections) = iter.next().expect("DD");
            assert_eq!(node.name, "DD");
            assert_eq!(node.flow_rate, 20);
            assert_eq!(connections, vec!["AA"]);

            let (node, connections) = iter.next().expect("II");
            assert_eq!(node.name, "II");
            assert_eq!(node.flow_rate, 0);
            assert_eq!(connections, vec!["AA", "JJ"]);

            let (node, connections) = iter.next().expect("JJ");
            assert_eq!(node.name, "JJ");
            assert_eq!(node.flow_rate, 21);
            assert_eq!(connections, vec!["II"]);

            assert_eq!(remaining_input, "");
        }
    }
}
