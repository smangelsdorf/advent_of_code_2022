use std::collections::BTreeMap;
use std::io::Read;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, not_line_ending, space1};
use nom::combinator::{eof, value};
use nom::multi::{many1, separated_list0};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::{IResult, Parser};

#[derive(Debug)]
struct ExpectedDirGotFile;

impl std::error::Error for ExpectedDirGotFile {}

impl std::fmt::Display for ExpectedDirGotFile {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("expected dir got file")
    }
}

// This implementation is an exercise in zero-copy parsing. The 'p lifetime stands for parsed -
// it's the lifetime of the `&str` which was borrowed from the input buffer in `main`. Instead of
// making lots of owned String values each with their own memory allocated, we take slices of the
// original and use them everywhere.
//
// That's not to say there are no allocations at all - there will definitely be allocations in the
// nested DirEnt structure, it's just that we don't copy the strings from the input.

#[derive(Debug, Eq, PartialEq)]
enum DirEnt<'p> {
    File {
        size: usize,
    },
    Dir {
        entries: BTreeMap<&'p str, DirEnt<'p>>,
    },
}

impl<'p> DirEnt<'p> {
    fn get_nested_mut<I>(&mut self, mut path: I) -> Result<&mut Self, ExpectedDirGotFile>
    where
        I: Iterator<Item = &'p str>,
    {
        match (self, path.next()) {
            (DirEnt::File { .. }, _) => Err(ExpectedDirGotFile),
            (DirEnt::Dir { entries }, Some(name)) => entries
                .entry(name)
                .or_insert_with(|| DirEnt::Dir {
                    entries: BTreeMap::default(),
                })
                .get_nested_mut(path),
            (s, None) => Ok(s),
        }
    }

    fn multi_insert<I>(&mut self, listing: I) -> Result<(), ExpectedDirGotFile>
    where
        I: Iterator<Item = LsEntry<'p>>,
    {
        match self {
            DirEnt::File { .. } => Err(ExpectedDirGotFile),
            DirEnt::Dir { entries } => {
                entries.extend(listing.map(|e| match e {
                    LsEntry::File { size, name } => (name, DirEnt::File { size }),
                    LsEntry::Dir { name } => (
                        name,
                        DirEnt::Dir {
                            entries: BTreeMap::default(),
                        },
                    ),
                }));
                Ok(())
            }
        }
    }

    fn visit_recursive_sizes(&self, f: &mut dyn FnMut(usize) -> ()) -> usize {
        match self {
            DirEnt::File { size } => *size,
            DirEnt::Dir { entries } => {
                let size = entries.values().map(|e| e.visit_recursive_sizes(f)).sum();
                f(size);
                size
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum LsEntry<'p> {
    File { size: usize, name: &'p str },
    Dir { name: &'p str },
}

#[derive(Debug, Eq, PartialEq)]
enum CdTarget<'p> {
    Root,
    Parent,
    Child(&'p str),
}

#[derive(Debug, Eq, PartialEq)]
enum Command<'p> {
    Ls { listing: Vec<LsEntry<'p>> },
    Cd { target: CdTarget<'p> },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    let (_remaining, commands) =
        parser::command_parser(&buffer).map_err(|e| e.map_input(str::to_owned))?;

    let mut root = DirEnt::Dir {
        entries: BTreeMap::default(),
    };
    let mut pwd = vec![];
    for command in commands {
        match command {
            Command::Cd {
                target: CdTarget::Root,
            } => {
                pwd.clear();
            }
            Command::Cd {
                target: CdTarget::Parent,
            } => {
                pwd.pop();
            }
            Command::Cd {
                target: CdTarget::Child(target),
            } => {
                pwd.push(target);
            }
            Command::Ls { listing } => {
                root.get_nested_mut(pwd.iter().copied())?
                    .multi_insert(listing.into_iter())?;
            }
        }
    }

    let mut total: usize = 0;
    root.visit_recursive_sizes(&mut |size| {
        if size <= 100000 {
            total += size
        }
    });
    println!("{}", total);

    Ok(())
}

mod parser {
    use super::*;

    fn base10_usize(input: &str) -> IResult<&str, usize> {
        digit1
            .map(|s| usize::from_str_radix(s, 10).unwrap())
            .parse(input)
    }

    fn raw_dirent_name(input: &str) -> IResult<&str, &str> {
        not_line_ending.map(str::trim_end).parse(input)
    }

    fn command_line<'a, P, O>(matcher: P) -> impl Parser<&'a str, O, nom::error::Error<&'a str>>
    where
        P: Parser<&'a str, O, nom::error::Error<&'a str>>,
    {
        preceded(tuple((tag("$"), space1)), matcher)
    }

    fn end_of_command<'p>() -> impl Parser<&'p str, (), nom::error::Error<&'p str>> {
        value((), alt((line_ending, eof)))
    }

    fn ls_entries(input: &str) -> IResult<&str, Vec<LsEntry>> {
        terminated(
            separated_list0(
                line_ending,
                alt((
                    preceded(tuple((tag("dir"), space1)), raw_dirent_name)
                        .map(|name| LsEntry::Dir { name }),
                    separated_pair(base10_usize, space1, raw_dirent_name)
                        .map(|(size, name)| LsEntry::File { size, name }),
                )),
            ),
            end_of_command(),
        )
        .parse(input)
    }

    fn ls_command(input: &str) -> IResult<&str, Command> {
        preceded(
            tuple((command_line(tag("ls")), end_of_command())),
            ls_entries,
        )
        .map(|listing| Command::Ls { listing })
        .parse(input)
    }

    fn cd_target(input: &str) -> IResult<&str, CdTarget> {
        raw_dirent_name
            .map(|s| match s {
                "/" => CdTarget::Root,
                ".." => CdTarget::Parent,
                s => CdTarget::Child(s),
            })
            .parse(input)
    }

    fn cd_command(input: &str) -> IResult<&str, Command> {
        terminated(
            preceded(tuple((command_line(tag("cd")), space1)), cd_target),
            end_of_command(),
        )
        .map(|target| Command::Cd { target })
        .parse(input)
    }

    pub(super) fn command_parser(input: &str) -> IResult<&str, Vec<Command>> {
        terminated(many1(alt((ls_command, cd_command))), eof).parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parser() {
            let r = command_parser(
                "$ cd /\n\
                 $ ls\n\
                 dir a\n\
                 1234 b\n\
                 $ cd a\n\
                 $ ls\n\
                 4321 c\n\
                 $ cd ..\n",
            );
            let (tail, parsed) = r.unwrap();
            assert_eq!("", tail);
            assert_eq!(
                vec![
                    Command::Cd {
                        target: CdTarget::Root
                    },
                    Command::Ls {
                        listing: vec![
                            LsEntry::Dir { name: "a" },
                            LsEntry::File {
                                name: "b",
                                size: 1234
                            }
                        ]
                    },
                    Command::Cd {
                        target: CdTarget::Child("a")
                    },
                    Command::Ls {
                        listing: vec![LsEntry::File {
                            name: "c",
                            size: 4321
                        }]
                    },
                    Command::Cd {
                        target: CdTarget::Parent
                    },
                ],
                parsed
            );
        }
    }
}
