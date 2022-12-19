// This is very slow to run but I've already spent way too much time
// on day 16. It's just a poor graph traversal implementation but I
// wasn't sure how to short-circuit it further.
//
// On my test input, it evaluates around 322 million states before
// completing.

use id_arena::Id;
use im::HashSet as ImHashSet;

use aoc::{parser::read_from_stdin_and_parse, MoreIter};
use graph::{ValveGraph, ValveNode, ValveNodeConnection};

#[derive(Debug, PartialEq, Clone)]
struct Valve {
    name: String,
    flow_rate: u64,
}

const START_VALVE: &'static str = "AA";
const TIME_LIMIT: u64 = 26;
const ACTOR_COUNT: usize = 2;

#[derive(Debug)]
struct State<'a, const N: usize> {
    actors: [(u64, &'a ValveNode); N],
    flow_rate: u64,
    released: u64,
    time_tallied: u64,
    visited: ImHashSet<Id<ValveNode>>,
}

impl<'a, const N: usize> State<'a, N> {
    fn new(node: &'a ValveNode) -> State<'a, N> {
        State {
            actors: [(0, node); N],
            flow_rate: 0,
            released: 0,
            time_tallied: 0,
            visited: ImHashSet::unit(node.id),
        }
    }

    fn can_continue(&self) -> bool {
        self.actors.iter().any(|(t, _)| t < &TIME_LIMIT)
    }

    fn is_valid(&self) -> bool {
        self.actors.iter().all(|(t, _)| t <= &TIME_LIMIT) && self.time_tallied == TIME_LIMIT
    }

    // Compute the final total for the `State`, assuming it doesn't move again.
    fn finish(&self) -> State<'a, N> {
        let &State {
            actors,
            mut flow_rate,
            mut released,
            mut time_tallied,
            ref visited,
        } = self;

        let mut actors = actors.clone();

        // Accrue for visited valves that haven't been added yet.
        for (t, node) in actors.iter_mut().filter(|(t, _)| t < &TIME_LIMIT) {
            let t = std::mem::replace(t, TIME_LIMIT);
            let rate = flow_rate;
            flow_rate += node.valve.flow_rate;
            released += (t - time_tallied) * rate;
            time_tallied = t;
        }

        released += (TIME_LIMIT - time_tallied) * flow_rate;

        State {
            actors,
            flow_rate,
            released,
            time_tallied: TIME_LIMIT,
            visited: visited.clone(),
        }
    }

    fn update(&self, conn: &ValveNodeConnection, new_node: &'a ValveNode) -> State<'a, N> {
        let &State {
            actors,
            flow_rate,
            released,
            time_tallied,
            ref visited,
        } = self;

        let release_steps = actors[0].0 - time_tallied;

        let mut actors = actors.clone();
        let new_time_spent = actors[0].0 + conn.cost + 1;
        let (_t, old_node) = std::mem::replace(&mut actors[0], (new_time_spent, new_node));

        actors.sort_by_key(|(a, _b)| *a);

        State {
            actors,
            flow_rate: flow_rate + old_node.valve.flow_rate,
            released: released + release_steps * flow_rate,
            time_tallied: time_tallied + release_steps,
            visited: visited.update(new_node.id),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = read_from_stdin_and_parse(parser::parse_input)?;

    let mut processed = 0;
    let mut max = State::<ACTOR_COUNT>::new(graph.start());
    let mut current = vec![State::<ACTOR_COUNT>::new(graph.start())];

    while let Some(state) = current.pop() {
        if state.can_continue() {
            let graph = &graph;
            let done = state.finish();

            processed += 1;
            if processed % 1_000_000 == 0 {
                println!("processed: {}", processed);
            }

            let (time_spent, node) = state.actors[0];
            let visited = state.visited.clone();

            let iter = node
                .connections
                .iter()
                .filter_map(move |conn| {
                    let node = graph.get(conn.target);
                    if visited.contains(&node.id) || time_spent + conn.cost + 1 > TIME_LIMIT {
                        None
                    } else {
                        Some(state.update(conn, node))
                    }
                })
                .once_if_empty(done);
            current.extend(iter);
        } else {
            let state = state.finish();

            if state.is_valid() && state.released > max.released {
                max = state;
            }
        }
    }

    let n = max.released;
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

#[cfg(test)]
mod tests {
    use crate::parser::parse_input;

    use super::*;

    #[test]
    fn test_state_update_1() {
        let input = "Valve AA has flow rate=0; tunnels lead to valves DD, II\n\
                     Valve DD has flow rate=20; tunnels lead to valves AA\n\
                     Valve II has flow rate=0; tunnels lead to valves AA, JJ\n\
                     Valve JJ has flow rate=21; tunnel leads to valve II, KK\n\
                     Valve KK has flow rate=5; tunnel leads to valve JJ";

        let (_, graph) = parse_input(input).unwrap();

        let state = State::<1>::new(graph.start());

        fn f<'a>(
            graph: &'a ValveGraph,
            node: &'a ValveNode,
            name: &str,
        ) -> (&'a ValveNodeConnection, &'a ValveNode) {
            node.connections
                .iter()
                .find_map(|c| {
                    let node = graph.get(c.target);
                    if node.valve.name == name {
                        Some((c, node))
                    } else {
                        None
                    }
                })
                .unwrap()
        }

        let (dd_conn, dd_node) = f(&graph, graph.start(), "DD");

        let state = state.update(dd_conn, dd_node);
        assert_eq!(
            state
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
            vec![(2u64, dd_node.id)]
        );

        assert_eq!(state.flow_rate, 0);
        assert_eq!(state.released, 0);
        assert_eq!(state.time_tallied, 0);
        assert_eq!(
            state.visited,
            [graph.start().id, dd_node.id].into_iter().collect()
        );

        let (jj_conn, jj_node) = f(&graph, dd_node, "JJ");

        let state = state.update(jj_conn, jj_node);
        assert_eq!(
            state
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
            vec![(6u64, jj_node.id)]
        );

        assert_eq!(state.flow_rate, 20);
        assert_eq!(state.released, 0);
        assert_eq!(state.time_tallied, 2);
        assert_eq!(
            state.visited,
            [graph.start().id, dd_node.id, jj_node.id]
                .into_iter()
                .collect()
        );

        let (kk_conn, kk_node) = f(&graph, jj_node, "KK");

        let state = state.update(kk_conn, kk_node);
        assert_eq!(
            state
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
            vec![(8u64, kk_node.id)]
        );

        assert_eq!(state.flow_rate, 41);
        assert_eq!(state.released, 80);
        assert_eq!(state.time_tallied, 6);
        assert_eq!(
            state.visited,
            [graph.start().id, dd_node.id, jj_node.id, kk_node.id]
                .into_iter()
                .collect()
        );

        let state = state.finish();

        assert_eq!(
            state
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
            vec![(TIME_LIMIT, kk_node.id)]
        );

        assert_eq!(state.flow_rate, 46);
        assert_eq!(state.released, 162 + (TIME_LIMIT - 8) * 46);
        assert_eq!(state.time_tallied, TIME_LIMIT);
        assert_eq!(
            state.visited,
            [graph.start().id, dd_node.id, jj_node.id, kk_node.id]
                .into_iter()
                .collect()
        );

        let state_idle_again = state.finish();

        assert_eq!(
            state
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
            state_idle_again
                .actors
                .iter()
                .map(|(t, c)| (*t, c.id))
                .collect::<Vec<_>>(),
        );

        assert_eq!(state.flow_rate, state_idle_again.flow_rate);
        assert_eq!(state.released, state_idle_again.released);
        assert_eq!(state.time_tallied, state_idle_again.time_tallied);
        assert_eq!(state.visited, state_idle_again.visited);
    }
}
