use std::collections::{HashMap, HashSet};

use tree::{Position, Quadtree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Elf {
    id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    N,
    S,
    W,
    E,
}

const DIRECTIONS: [Direction; 4] = [Direction::N, Direction::S, Direction::W, Direction::E];

pub fn main() {
    let mut elf_id = 0;

    let bytes = std::io::stdin()
        .lines()
        .map(Result::unwrap)
        .map(|line| line.bytes().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let width = bytes[0].len();
    let height = bytes.len();

    let mut tree = Quadtree::new((
        Position { x: -100, y: -100 },
        Position {
            x: width as i64 + 100,
            y: height as i64 + 100,
        },
    ));

    let mut elves = HashMap::new();

    for (y, line) in bytes.iter().enumerate() {
        for (x, &byte) in line.iter().enumerate() {
            if byte == b'#' {
                let position = Position {
                    x: x as i64,
                    y: y as i64,
                };

                tree.insert(position, Elf { id: elf_id });
                elves.insert(elf_id, position);
                elf_id += 1;
            }
        }
    }

    for i in 0.. {
        let directions = DIRECTIONS
            .iter()
            .cycle()
            .skip(i)
            .take(4)
            .collect::<Vec<_>>();

        let mut intentions = elves
            .iter()
            .filter_map(|(&id, &pos)| {
                let bounds = (
                    Position {
                        x: pos.x - 1,
                        y: pos.y - 1,
                    },
                    Position {
                        x: pos.x + 1,
                        y: pos.y + 1,
                    },
                );

                let neighbours: HashSet<_> = tree
                    .query(bounds)
                    .map(|(p, _)| p)
                    .filter(|p| *p != pos)
                    .collect::<HashSet<_>>();

                if neighbours.is_empty() {
                    return None;
                }

                for dir in &directions {
                    let (a, b, c) = match dir {
                        Direction::N => {
                            let pos = Position {
                                x: pos.x,
                                y: pos.y - 1,
                            };

                            (
                                Position {
                                    x: pos.x - 1,
                                    ..pos
                                },
                                pos,
                                Position {
                                    x: pos.x + 1,
                                    ..pos
                                },
                            )
                        }
                        Direction::S => {
                            let pos = Position {
                                x: pos.x,
                                y: pos.y + 1,
                            };

                            (
                                Position {
                                    x: pos.x - 1,
                                    ..pos
                                },
                                pos,
                                Position {
                                    x: pos.x + 1,
                                    ..pos
                                },
                            )
                        }
                        Direction::W => {
                            let pos = Position {
                                x: pos.x - 1,
                                y: pos.y,
                            };

                            (
                                Position {
                                    y: pos.y - 1,
                                    ..pos
                                },
                                pos,
                                Position {
                                    y: pos.y + 1,
                                    ..pos
                                },
                            )
                        }
                        Direction::E => {
                            let pos = Position {
                                x: pos.x + 1,
                                y: pos.y,
                            };

                            (
                                Position {
                                    y: pos.y - 1,
                                    ..pos
                                },
                                pos,
                                Position {
                                    y: pos.y + 1,
                                    ..pos
                                },
                            )
                        }
                    };

                    if !neighbours.contains(&a)
                        && !neighbours.contains(&b)
                        && !neighbours.contains(&c)
                    {
                        return Some((id, b));
                    }
                }

                None
            })
            .collect::<HashMap<_, _>>();

        let intention_count: HashMap<_, _> =
            intentions
                .iter()
                .map(|(_, &pos)| pos)
                .fold(HashMap::new(), |mut map, pos| {
                    *map.entry(pos).or_insert(0) += 1;
                    map
                });

        intentions.retain(|_, pos| intention_count.get(pos).copied().unwrap_or(0) == 1);

        if intentions.is_empty() {
            println!("finished at {}", i + 1);
            break;
        }

        for (&id, &pos) in intentions.iter() {
            let old_pos = elves.insert(id, pos).unwrap();
            tree.remove(old_pos, |e| e.id == id);
            tree.insert(pos, Elf { id });
        }
    }

    let (min, max) = elves.values().fold(
        (
            Position {
                x: i64::MAX,
                y: i64::MAX,
            },
            Position {
                x: i64::MIN,
                y: i64::MIN,
            },
        ),
        |(min, max), pos| {
            (
                Position {
                    x: min.x.min(pos.x),
                    y: min.y.min(pos.y),
                },
                Position {
                    x: max.x.max(pos.x),
                    y: max.y.max(pos.y),
                },
            )
        },
    );

    let width = max.x - min.x + 1;
    let height = max.y - min.y + 1;

    println!("{}x{} = {}", width, height, width * height);
    println!(
        "{} - {} elves = {}",
        width * height,
        elves.len(),
        width * height - elves.len() as i64
    );
}

mod tree {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Position {
        pub x: i64,
        pub y: i64,
    }

    #[derive(Debug)]
    pub(super) struct Quadtree<T> {
        root: Node<T>,
    }

    impl<T> Quadtree<T> {
        pub(super) fn new(bounds: (Position, Position)) -> Self {
            Self {
                root: Node::new_leaf(bounds),
            }
        }

        pub(super) fn insert(&mut self, position: Position, object: T) {
            self.root.insert(position, object);
        }

        pub(super) fn remove<F>(&mut self, position: Position, f: F)
        where
            F: FnMut(&T) -> bool,
        {
            self.root.remove(position, f);
        }

        #[allow(dead_code)]
        pub(super) fn iter(&self) -> impl Iterator<Item = (Position, &T)> {
            QuadtreeIter {
                stack: vec![(&self.root, 0)],
            }
        }

        pub(super) fn query(
            &self,
            bounds: (Position, Position),
        ) -> impl Iterator<Item = (Position, &T)> {
            QuadtreeQuery {
                stack: vec![(&self.root, 0)],
                bounds,
            }
        }
    }

    #[derive(Debug)]
    #[repr(usize)]
    enum Quadrant {
        NW = 0,
        NE = 1,
        SW = 2,
        SE = 3,
    }

    const MAX_LEAF_CHILDREN: usize = 16;

    #[derive(Debug)]
    enum Node<T> {
        Leaf {
            bounds: (Position, Position),
            objects: Vec<(Position, T)>,
        },
        Branch {
            bounds: (Position, Position),
            pivot: Position,
            children: Box<[Node<T>; 4]>,
        },
    }

    impl<T> Node<T> {
        fn new_leaf(bounds: (Position, Position)) -> Self {
            Self::Leaf {
                bounds,
                objects: Vec::new(),
            }
        }

        fn insert(&mut self, position: Position, object: T) {
            match self {
                Self::Leaf { bounds, objects } => {
                    objects.push((position, object));

                    if objects.len() > MAX_LEAF_CHILDREN {
                        *self = Self::into_branch(*bounds, objects.drain(..));
                    }
                }
                Self::Branch {
                    pivot, children, ..
                } => {
                    let quadrant = if position.x < pivot.x {
                        if position.y < pivot.y {
                            Quadrant::NW
                        } else {
                            Quadrant::SW
                        }
                    } else if position.y < pivot.y {
                        Quadrant::NE
                    } else {
                        Quadrant::SE
                    };

                    children[quadrant as usize].insert(position, object);
                }
            }
        }

        fn remove<F>(&mut self, position: Position, mut f: F)
        where
            F: FnMut(&T) -> bool,
        {
            match self {
                Self::Leaf { objects, .. } => {
                    if let Some(i) = objects
                        .iter()
                        .position(|(pos, obj)| *pos == position && f(obj))
                    {
                        objects.swap_remove(i);
                    }
                }
                Self::Branch {
                    pivot, children, ..
                } => {
                    let quadrant = if position.x < pivot.x {
                        if position.y < pivot.y {
                            Quadrant::NW
                        } else {
                            Quadrant::SW
                        }
                    } else if position.y < pivot.y {
                        Quadrant::NE
                    } else {
                        Quadrant::SE
                    };

                    children[quadrant as usize].remove(position, f);
                }
            }
        }

        fn into_branch<I>(bounds: (Position, Position), objects: I) -> Self
        where
            I: Iterator<Item = (Position, T)>,
        {
            let (min, max) = bounds;
            let pivot = Position {
                x: (min.x + max.x) / 2,
                y: (min.y + max.y) / 2,
            };

            let (w, e) = objects
                .into_iter()
                .partition::<Vec<_>, _>(|(pos, _)| pos.x < pivot.x);
            let (nw, sw) = w.into_iter().partition(|(pos, _)| pos.y < pivot.y);
            let (ne, se) = e.into_iter().partition(|(pos, _)| pos.y < pivot.y);

            let nw_bounds = (min, pivot);
            let ne_bounds = (
                Position {
                    x: pivot.x,
                    y: min.y,
                },
                Position {
                    x: max.x,
                    y: pivot.y,
                },
            );
            let sw_bounds = (
                Position {
                    x: min.x,
                    y: pivot.y,
                },
                Position {
                    x: pivot.x,
                    y: max.y,
                },
            );
            let se_bounds = (pivot, max);

            let children = Box::new([
                Node::Leaf {
                    bounds: nw_bounds,
                    objects: nw,
                },
                Node::Leaf {
                    bounds: ne_bounds,
                    objects: ne,
                },
                Node::Leaf {
                    bounds: sw_bounds,
                    objects: sw,
                },
                Node::Leaf {
                    bounds: se_bounds,
                    objects: se,
                },
            ]);

            Self::Branch {
                bounds,
                pivot,
                children,
            }
        }
    }

    struct QuadtreeIter<'a, T> {
        stack: Vec<(&'a Node<T>, usize)>,
    }

    impl<'a, T> Iterator for QuadtreeIter<'a, T> {
        type Item = (Position, &'a T);

        fn next(&mut self) -> Option<Self::Item> {
            while let Some((node, index)) = self.stack.pop() {
                match node {
                    Node::Leaf { objects, .. } => {
                        if let Some((position, object)) = objects.get(index) {
                            self.stack.push((node, index + 1));
                            return Some((*position, object));
                        }
                    }
                    Node::Branch { children, .. } => {
                        if let Some(child) = children.get(index) {
                            self.stack.push((node, index + 1));
                            self.stack.push((child, 0));
                        }
                    }
                }

                return self.next();
            }

            None
        }
    }

    struct QuadtreeQuery<'a, T> {
        stack: Vec<(&'a Node<T>, usize)>,
        bounds: (Position, Position),
    }

    impl<'a, T> Iterator for QuadtreeQuery<'a, T> {
        type Item = (Position, &'a T);

        fn next(&mut self) -> Option<Self::Item> {
            while let Some((node, index)) = self.stack.pop() {
                match node {
                    Node::Leaf { objects, .. } => {
                        if let Some((position, object)) = objects.get(index) {
                            self.stack.push((node, index + 1));

                            if position.x <= self.bounds.1.x
                                && position.y <= self.bounds.1.y
                                && position.x >= self.bounds.0.x
                                && position.y >= self.bounds.0.y
                            {
                                return Some((*position, object));
                            }
                        }
                    }
                    Node::Branch {
                        bounds, children, ..
                    } => {
                        if bounds.0.x <= self.bounds.1.x
                            && bounds.0.y <= self.bounds.1.y
                            && bounds.1.x >= self.bounds.0.x
                            && bounds.1.y >= self.bounds.0.y
                        {
                            if let Some(child) = children.get(index) {
                                self.stack.push((node, index + 1));
                                self.stack.push((child, 0));
                            }
                        }
                    }
                }

                return self.next();
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use itertools::Itertools;

    use super::tree::*;

    #[test]
    fn test_quadtree() {
        let mut tree = Quadtree::new((Position { x: 0, y: 0 }, Position { x: 200, y: 200 }));

        let items = (b'A'..b'Z')
            .cartesian_product(b'A'..b'Z')
            .map(|(x, y)| {
                (
                    Position {
                        x: x as i64,
                        y: y as i64,
                    },
                    String::from_utf8(vec![x, y]).unwrap(),
                )
            })
            .collect::<Vec<_>>();

        for (position, object) in items.iter() {
            tree.insert(*position, object.clone());
        }

        assert_eq!(
            tree.iter()
                .map(|(a, b)| (a, b.clone()))
                .collect::<HashMap<Position, String>>(),
            items.iter().cloned().collect()
        );

        assert_eq!(
            tree.query((Position { x: 0, y: 0 }, Position { x: 100, y: 100 }))
                .map(|(a, b)| (a, b.clone()))
                .collect::<HashMap<Position, String>>(),
            items.into_iter().collect()
        );

        for i in b'A'..b'Y' {
            assert_eq!(
                tree.query((
                    Position {
                        x: i as i64,
                        y: i as i64
                    },
                    Position {
                        x: i as i64 + 1,
                        y: i as i64 + 1
                    }
                ))
                .map(|(a, b)| (a, b.as_str()))
                .collect::<HashMap<Position, &str>>(),
                [
                    (
                        Position {
                            x: i as i64,
                            y: i as i64
                        },
                        std::str::from_utf8(&[i, i]).unwrap()
                    ),
                    (
                        Position {
                            x: i as i64,
                            y: i as i64 + 1
                        },
                        std::str::from_utf8(&[i, i + 1]).unwrap()
                    ),
                    (
                        Position {
                            x: i as i64 + 1,
                            y: i as i64
                        },
                        std::str::from_utf8(&[i + 1, i]).unwrap()
                    ),
                    (
                        Position {
                            x: i as i64 + 1,
                            y: i as i64 + 1
                        },
                        std::str::from_utf8(&[i + 1, i + 1]).unwrap()
                    ),
                ]
                .into_iter()
                .collect()
            )
        }
    }
}
