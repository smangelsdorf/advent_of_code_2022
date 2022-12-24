// This is slow, but it works!

use std::collections::BTreeMap;

use aoc::parser::read_from_stdin_and_parse;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State<'a> {
    blueprint: &'a Blueprint,
    ore: u64,
    clay: u64,
    obsidian: u64,
    geode: u64,
    ore_robot: u64,
    clay_robot: u64,
    obsidian_robot: u64,
    geode_robot: u64,
    time: u64,
    next_action: Action,
}

impl<'a> State<'a> {
    fn initial_iterator(blueprint: &'a Blueprint) -> StateStepIter<'a> {
        let state = Self {
            blueprint,
            ore: 0,
            clay: 0,
            obsidian: 0,
            geode: 0,
            ore_robot: 1,
            clay_robot: 0,
            obsidian_robot: 0,
            geode_robot: 0,
            time: 0,
            next_action: Action::BuildGeodeRobot,
        };

        StateStepIter::Continue {
            state,
            build_ore_robot: state.future_legal(Action::BuildOreRobot),
            build_clay_robot: state.future_legal(Action::BuildClayRobot),
            build_obsidian_robot: state.future_legal(Action::BuildObsidianRobot),
            build_geode_robot: state.future_legal(Action::BuildGeodeRobot),
        }
    }

    fn spend(&mut self, cost: Cost) {
        self.ore -= cost.ore;
        self.clay -= cost.clay;
        self.obsidian -= cost.obsidian;
    }

    fn accrue(&mut self) {
        self.ore += self.ore_robot;
        self.clay += self.clay_robot;
        self.obsidian += self.obsidian_robot;
        self.geode += self.geode_robot;
    }

    fn tick_until_action(&mut self) -> StateStepIter<'a> {
        while !self.legal(self.next_action) {
            self.time += 1;
            self.accrue();

            if self.done() {
                return StateStepIter::Done { state: Some(*self) };
            }
        }

        self.spend(self.blueprint.cost_of(self.next_action));

        self.time += 1;
        self.accrue();

        let target = match self.next_action {
            Action::BuildOreRobot => &mut self.ore_robot,
            Action::BuildClayRobot => &mut self.clay_robot,
            Action::BuildObsidianRobot => &mut self.obsidian_robot,
            Action::BuildGeodeRobot => &mut self.geode_robot,
        };

        *target += 1;

        if self.done() {
            StateStepIter::Done { state: Some(*self) }
        } else {
            StateStepIter::Continue {
                state: *self,
                build_ore_robot: self.future_legal(Action::BuildOreRobot),
                build_clay_robot: self.future_legal(Action::BuildClayRobot),
                build_obsidian_robot: self.future_legal(Action::BuildObsidianRobot),
                build_geode_robot: self.future_legal(Action::BuildGeodeRobot),
            }
        }
    }

    fn max_costs(&self) -> Cost {
        let (max_ore_cost, max_clay_cost, max_obsidian_cost) =
            self.blueprint
                .costs()
                .fold((0, 0, 0), |(max_ore, max_clay, max_obsidian), cost| {
                    (
                        max_ore.max(cost.ore),
                        max_clay.max(cost.clay),
                        max_obsidian.max(cost.obsidian),
                    )
                });

        Cost {
            ore: max_ore_cost,
            clay: max_clay_cost,
            obsidian: max_obsidian_cost,
        }
    }

    fn legal(&self, action: Action) -> bool {
        let cost = self.blueprint.cost_of(action);
        self.ore >= cost.ore && self.clay >= cost.clay && self.obsidian >= cost.obsidian
    }

    // Prune future actions that are impossible or clearly suboptimal.
    fn future_legal(&self, action: Action) -> bool {
        let cost = self.blueprint.cost_of(action);
        let max_costs = self.max_costs();

        let remaining_time = MAX_TIME - self.time;

        match action {
            Action::BuildOreRobot if self.ore_robot > max_costs.ore => return false,
            Action::BuildClayRobot if self.clay_robot > max_costs.clay => return false,
            Action::BuildObsidianRobot if self.obsidian_robot > max_costs.obsidian => return false,
            _ => {}
        }

        // If we'll accrue enough resources before the time limit, we can do it.
        self.ore + remaining_time * self.ore_robot >= cost.ore
            && self.clay + remaining_time * self.clay_robot >= cost.clay
            && self.obsidian + remaining_time * self.obsidian_robot >= cost.obsidian
    }

    fn done(&self) -> bool {
        self.time >= MAX_TIME
    }
}

// Custom iterator to avoid allocating a Vec for intermediate steps.
#[derive(Debug, Clone, Copy)]
enum StateStepIter<'a> {
    Continue {
        state: State<'a>,
        build_ore_robot: bool,
        build_clay_robot: bool,
        build_obsidian_robot: bool,
        build_geode_robot: bool,
    },
    Done {
        state: Option<State<'a>>,
    },
}

impl<'a> Iterator for StateStepIter<'a> {
    type Item = State<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StateStepIter::Done { state } => state.take(),

            StateStepIter::Continue {
                state,
                build_ore_robot: ref mut b,
                ..
            } if *b => {
                *b = false;
                Some(State {
                    next_action: Action::BuildOreRobot,
                    ..*state
                })
            }

            StateStepIter::Continue {
                state,
                build_clay_robot: ref mut b,
                ..
            } if *b => {
                *b = false;
                Some(State {
                    next_action: Action::BuildClayRobot,
                    ..*state
                })
            }

            StateStepIter::Continue {
                state,
                build_obsidian_robot: ref mut b,
                ..
            } if *b => {
                *b = false;
                Some(State {
                    next_action: Action::BuildObsidianRobot,
                    ..*state
                })
            }

            StateStepIter::Continue {
                state,
                build_geode_robot: ref mut b,
                ..
            } if *b => {
                *b = false;
                Some(State {
                    next_action: Action::BuildGeodeRobot,
                    ..*state
                })
            }

            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    BuildOreRobot,
    BuildClayRobot,
    BuildObsidianRobot,
    BuildGeodeRobot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Cost {
    ore: u64,
    clay: u64,
    obsidian: u64,
}

impl std::ops::Add<Cost> for Cost {
    type Output = Cost;

    fn add(self, other: Cost) -> Cost {
        Cost {
            ore: self.ore + other.ore,
            clay: self.clay + other.clay,
            obsidian: self.obsidian + other.obsidian,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Blueprint {
    ore_robot_cost: Cost,
    clay_robot_cost: Cost,
    obsidian_robot_cost: Cost,
    geode_robot_cost: Cost,
}

impl Blueprint {
    fn costs(&self) -> impl Iterator<Item = Cost> {
        std::iter::once(self.ore_robot_cost)
            .chain(std::iter::once(self.clay_robot_cost))
            .chain(std::iter::once(self.obsidian_robot_cost))
            .chain(std::iter::once(self.geode_robot_cost))
    }

    fn cost_of(&self, action: Action) -> Cost {
        match action {
            Action::BuildOreRobot => self.ore_robot_cost,
            Action::BuildClayRobot => self.clay_robot_cost,
            Action::BuildObsidianRobot => self.obsidian_robot_cost,
            Action::BuildGeodeRobot => self.geode_robot_cost,
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut blueprints = read_from_stdin_and_parse(parser::parse_input)?;
    blueprints.truncate(3);

    let mut best = BTreeMap::new();

    best.par_extend(
        blueprints
            .into_par_iter()
            .map(|(i, blueprint)| (i, simulate(i, blueprint))),
    );
    println!("{:?}", best);

    let n = best.into_iter().map(|(_i, geode)| geode).product::<u64>();
    println!("{}", n);

    Ok(())
}

const MAX_TIME: u64 = 32;
fn simulate(i: u64, blueprint: Blueprint) -> u64 {
    println!("{}: {:?}", i, blueprint);
    let mut current = State::initial_iterator(&blueprint).collect::<Vec<_>>();
    let mut max = None;

    while let Some(mut state) = current.pop() {
        if state.done() {
            if let Some(max) = max.as_mut() {
                if state.geode > *max {
                    *max = state.geode;
                }
            } else {
                max = Some(state.geode);
            }

            continue;
        }

        current.extend(state.tick_until_action())
    }

    println!("{}: done ({:?})", i, max);
    max.unwrap()
}

mod parser {
    use super::*;

    use aoc::parser::base10_numeric;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{line_ending, space1},
        combinator::map_opt,
        multi::{many1, separated_list1},
        sequence::{delimited, terminated, tuple},
        IResult, Parser,
    };

    fn cost(input: &str) -> IResult<&str, Cost> {
        map_opt(
            separated_list1(
                tag(" and "),
                alt((
                    terminated(base10_numeric, tag(" ore")).map(|ore| Cost {
                        ore,
                        ..Default::default()
                    }),
                    terminated(base10_numeric, tag(" clay")).map(|clay| Cost {
                        clay,
                        ..Default::default()
                    }),
                    terminated(base10_numeric, tag(" obsidian")).map(|obsidian| Cost {
                        obsidian,
                        ..Default::default()
                    }),
                )),
            ),
            |costs| costs.into_iter().reduce(std::ops::Add::add),
        )
        .parse(input)
    }

    fn ore_robot_cost(input: &str) -> IResult<&str, Cost> {
        delimited(tag("Each ore robot costs "), cost, tag(".")).parse(input)
    }

    fn clay_robot_cost(input: &str) -> IResult<&str, Cost> {
        delimited(tag("Each clay robot costs "), cost, tag(".")).parse(input)
    }

    fn obsidian_robot_cost(input: &str) -> IResult<&str, Cost> {
        delimited(tag("Each obsidian robot costs "), cost, tag(".")).parse(input)
    }

    fn geode_robot_cost(input: &str) -> IResult<&str, Cost> {
        delimited(tag("Each geode robot costs "), cost, tag(".")).parse(input)
    }

    fn spacer(input: &str) -> IResult<&str, ()> {
        many1(alt((line_ending, space1))).map(|_| ()).parse(input)
    }

    fn blueprint(input: &str) -> IResult<&str, (u64, Blueprint)> {
        tuple((
            terminated(
                delimited(tag("Blueprint "), base10_numeric, tag(":")),
                spacer,
            ),
            terminated(ore_robot_cost, spacer),
            terminated(clay_robot_cost, spacer),
            terminated(obsidian_robot_cost, spacer),
            geode_robot_cost,
        ))
        .map(
            |(id, ore_robot_cost, clay_robot_cost, obsidian_robot_cost, geode_robot_cost)| {
                (
                    id,
                    Blueprint {
                        ore_robot_cost,
                        clay_robot_cost,
                        obsidian_robot_cost,
                        geode_robot_cost,
                    },
                )
            },
        )
        .parse(input)
    }

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<(u64, Blueprint)>> {
        separated_list1(many1(line_ending), blueprint).parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let input = "\
        Blueprint 1:\n\
        Each ore robot costs 4 ore.\n\
        Each clay robot costs 2 ore.\n\
        Each obsidian robot costs 3 ore and 14 clay.\n\
        Each geode robot costs 2 ore and 7 obsidian.\n\
        \n\
        Blueprint 2:\n\
        Each ore robot costs 2 ore.\n\
        Each clay robot costs 3 ore.\n\
        Each obsidian robot costs 3 ore and 8 clay.\n\
        Each geode robot costs 3 ore and 12 obsidian.\n\
        ";

        let expected = vec![
            (
                1,
                Blueprint {
                    ore_robot_cost: Cost {
                        ore: 4,
                        ..Default::default()
                    },
                    clay_robot_cost: Cost {
                        ore: 2,
                        ..Default::default()
                    },
                    obsidian_robot_cost: Cost {
                        ore: 3,
                        clay: 14,
                        ..Default::default()
                    },
                    geode_robot_cost: Cost {
                        ore: 2,
                        obsidian: 7,
                        ..Default::default()
                    },
                },
            ),
            (
                2,
                Blueprint {
                    ore_robot_cost: Cost {
                        ore: 2,
                        ..Default::default()
                    },
                    clay_robot_cost: Cost {
                        ore: 3,
                        ..Default::default()
                    },
                    obsidian_robot_cost: Cost {
                        ore: 3,
                        clay: 8,
                        ..Default::default()
                    },
                    geode_robot_cost: Cost {
                        ore: 3,
                        obsidian: 12,
                        ..Default::default()
                    },
                },
            ),
        ];

        assert_eq!(expected, parser::parse_input(input).unwrap().1);
    }

    #[test]
    fn test_state_spend() {
        let blueprint = Blueprint {
            ore_robot_cost: Cost {
                ore: 4,
                ..Default::default()
            },
            clay_robot_cost: Cost {
                ore: 2,
                ..Default::default()
            },
            obsidian_robot_cost: Cost {
                ore: 3,
                clay: 14,
                ..Default::default()
            },
            geode_robot_cost: Cost {
                ore: 2,
                obsidian: 7,
                ..Default::default()
            },
        };

        let mut state = State::initial_iterator(&blueprint).next().unwrap();

        state.accrue();
        state.accrue();
        state.accrue();
        state.accrue();

        assert_eq!(state.ore, 4);

        state.spend(Cost {
            ore: 4,
            ..Default::default()
        });

        assert_eq!(state.ore, 0);
    }

    #[test]
    fn test_state_max_costs() {
        let blueprint = Blueprint {
            ore_robot_cost: Cost {
                ore: 4,
                ..Default::default()
            },
            clay_robot_cost: Cost {
                ore: 2,
                ..Default::default()
            },
            obsidian_robot_cost: Cost {
                ore: 3,
                clay: 14,
                ..Default::default()
            },
            geode_robot_cost: Cost {
                ore: 2,
                obsidian: 7,
                ..Default::default()
            },
        };

        let state = State::initial_iterator(&blueprint).next().unwrap();

        assert_eq!(
            state.max_costs(),
            Cost {
                ore: 4,
                clay: 14,
                obsidian: 7,
            }
        );
    }
}
