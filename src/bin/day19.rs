use std::collections::BTreeMap;

use aoc::parser::read_from_stdin_and_parse;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State<'a> {
    blueprint: &'a Blueprint,
    ore: u16,
    clay: u16,
    obsidian: u16,
    geode: u16,
    ore_robot: u16,
    clay_robot: u16,
    obsidian_robot: u16,
    geode_robot: u16,
    time: u16,
}

impl<'a> State<'a> {
    fn new(blueprint: &'a Blueprint) -> Self {
        Self {
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

    fn tick(&mut self, action: Action) {
        self.spend(self.blueprint.cost_of(action));

        self.time += 1;
        self.accrue();

        match action {
            Action::BuildOreRobot => {
                self.ore_robot += 1;
            }
            Action::BuildClayRobot => {
                self.clay_robot += 1;
            }
            Action::BuildObsidianRobot => {
                self.obsidian_robot += 1;
            }
            Action::BuildGeodeRobot => {
                self.geode_robot += 1;
            }
            Action::DoNothing => {}
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

    fn useful(&self, action: Action) -> bool {
        if !self.legal(action) {
            return false;
        }

        let max_costs = self.max_costs();
        match action {
            Action::BuildOreRobot => self.ore_robot < max_costs.ore,
            Action::BuildClayRobot => self.clay_robot < max_costs.clay,
            Action::BuildObsidianRobot => self.obsidian_robot < max_costs.obsidian,
            Action::BuildGeodeRobot => true,
            // No reason to do nothing if we can build a geode robot
            Action::DoNothing => !self.legal(Action::BuildGeodeRobot),
        }
    }

    fn done(&self) -> bool {
        self.time >= MAX_TIME
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    BuildOreRobot,
    BuildClayRobot,
    BuildObsidianRobot,
    BuildGeodeRobot,
    DoNothing,
}

impl Action {
    fn all() -> impl Iterator<Item = Action> {
        std::iter::once(Action::BuildOreRobot)
            .chain(std::iter::once(Action::BuildClayRobot))
            .chain(std::iter::once(Action::BuildObsidianRobot))
            .chain(std::iter::once(Action::BuildGeodeRobot))
            .chain(std::iter::once(Action::DoNothing))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Cost {
    ore: u16,
    clay: u16,
    obsidian: u16,
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
            Action::DoNothing => Cost::default(),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blueprints = read_from_stdin_and_parse(parser::parse_input)?;

    let mut best = BTreeMap::new();

    best.par_extend(
        blueprints
            .into_par_iter()
            .map(|(i, blueprint)| (i, simulate(i, blueprint))),
    );

    let n = best.into_iter().map(|(i, geode)| (i * geode)).sum::<u16>();
    println!("{}", n);

    Ok(())
}

const MAX_TIME: u16 = 24;
fn simulate(i: u16, blueprint: Blueprint) -> u16 {
    println!("{}: {:?}", i, blueprint);
    let mut current = vec![State::new(&blueprint)];
    let mut max = None;

    while let Some(state) = current.pop() {
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

        for action in Action::all() {
            if state.useful(action) {
                let mut next = state.clone();
                next.tick(action);
                current.push(next);
            }
        }
    }

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

    fn blueprint(input: &str) -> IResult<&str, (u16, Blueprint)> {
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

    pub(super) fn parse_input(input: &str) -> IResult<&str, Vec<(u16, Blueprint)>> {
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

        let mut state = State::new(&blueprint);

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
    fn test_state_useful() {
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

        let mut state = State::new(&blueprint);
        assert!(!state.useful(Action::BuildOreRobot));
        assert!(!state.useful(Action::BuildClayRobot));
        assert!(!state.useful(Action::BuildObsidianRobot));
        assert!(!state.useful(Action::BuildGeodeRobot));
        assert!(state.useful(Action::DoNothing));

        state.ore = 4;
        assert!(state.useful(Action::BuildOreRobot));
        assert!(state.useful(Action::BuildClayRobot));
        assert!(!state.useful(Action::BuildObsidianRobot));
        assert!(!state.useful(Action::BuildGeodeRobot));
        assert!(state.useful(Action::DoNothing));

        state.clay = 20;
        assert!(state.useful(Action::BuildOreRobot));
        assert!(state.useful(Action::BuildClayRobot));
        assert!(state.useful(Action::BuildObsidianRobot));
        assert!(!state.useful(Action::BuildGeodeRobot));
        assert!(state.useful(Action::DoNothing));

        state.obsidian = 10;
        assert!(state.useful(Action::BuildOreRobot));
        assert!(state.useful(Action::BuildClayRobot));
        assert!(state.useful(Action::BuildObsidianRobot));
        assert!(state.useful(Action::BuildGeodeRobot));
        assert!(!state.useful(Action::DoNothing));

        state.ore_robot = 10;
        assert!(!state.useful(Action::BuildOreRobot));
        assert!(state.useful(Action::BuildClayRobot));
        assert!(state.useful(Action::BuildObsidianRobot));
        assert!(state.useful(Action::BuildGeodeRobot));

        state.clay_robot = 15;
        assert!(!state.useful(Action::BuildOreRobot));
        assert!(!state.useful(Action::BuildClayRobot));
        assert!(state.useful(Action::BuildObsidianRobot));
        assert!(state.useful(Action::BuildGeodeRobot));

        state.obsidian_robot = 10;
        assert!(!state.useful(Action::BuildOreRobot));
        assert!(!state.useful(Action::BuildClayRobot));
        assert!(!state.useful(Action::BuildObsidianRobot));
        assert!(state.useful(Action::BuildGeodeRobot));

        state.geode_robot = 10;
        assert!(!state.useful(Action::BuildOreRobot));
        assert!(!state.useful(Action::BuildClayRobot));
        assert!(!state.useful(Action::BuildObsidianRobot));
        assert!(state.useful(Action::BuildGeodeRobot));
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

        let state = State::new(&blueprint);

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
