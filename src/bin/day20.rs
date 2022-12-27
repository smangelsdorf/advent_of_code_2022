use aoc::parser::{base10_numeric, read_from_stdin_and_parse};
use nom::{character::complete::line_ending, multi::separated_list1, IResult, Parser};

use collection::RelocationVec;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vec = read_from_stdin_and_parse(parse_input)?;
    let mut vec = vec
        .into_iter()
        .map(|i| (i, false))
        .collect::<RelocationVec<_>>();

    mix(&mut vec, noop);

    let mut pos = vec.start();
    while vec.get(&pos).map(|&(value, _)| value) != Some(0) {
        pos = vec.advance(pos, 1);
    }

    println!("Found 0 at {:?}", pos);

    let sum = (1..=3)
        .map(|i| {
            vec.get(&vec.advance(pos, i * 1000))
                .map(|&(value, _)| value)
                .expect("always a value")
        })
        .sum::<i64>();

    println!("Sum: {}", sum);

    Ok(())
}

fn noop<const CHUNK_SIZE: usize>(_vec: &RelocationVec<(i64, bool), CHUNK_SIZE>) {}

fn mix<F, const CHUNK_SIZE: usize>(vec: &mut RelocationVec<(i64, bool), CHUNK_SIZE>, mut f: F)
where
    F: FnMut(&RelocationVec<(i64, bool), CHUNK_SIZE>),
{
    let mut remaining = vec.len();
    let mut pos = vec.start();

    // Inspect function used by the test cases.
    f(&vec);

    while remaining > 0 {
        let &(value, relocated) = vec.get(&pos).unwrap();

        if !relocated {
            let target = vec.relocate(pos, value);
            vec.get_mut(&target).unwrap().1 = true;
            remaining -= 1;

            f(&vec);
        }

        pos = vec.advance(pos, 1);
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<i64>> {
    separated_list1(line_ending, base10_numeric).parse(input)
}

mod collection {
    use std::fmt::Debug;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Position {
        chunk: usize,
        pos: usize,
    }

    impl Position {
        fn wrapping_succ(self, chunk_size: usize, num_chunks: usize) -> Self {
            let Position { mut chunk, mut pos } = self;

            // Move to the next slot, wrapping at the end.
            if pos >= chunk_size - 1 {
                if chunk >= num_chunks - 1 {
                    chunk = 0;
                    pos = 0;
                } else {
                    chunk += 1;
                    pos = 0;
                }
            } else {
                pos += 1;
            }

            Position { chunk, pos }
        }
    }

    // Collection with 50% occupancy that allows items to be moved around efficiently.
    #[derive(Debug)]
    pub(super) struct RelocationVec<T, const CHUNK_SIZE: usize = 32> {
        vec: Vec<[Option<T>; CHUNK_SIZE]>,

        // Expensive to compute, never changes.
        len: usize,
    }

    impl<T: Debug, const CHUNK_SIZE: usize> RelocationVec<T, CHUNK_SIZE> {
        pub(super) fn start(&self) -> Position {
            let first = Position { chunk: 0, pos: 0 };

            self.advance(first, 0)
        }

        pub(super) fn len(&self) -> usize {
            self.len
        }

        pub(super) fn get(&self, position: &Position) -> Option<&T> {
            self.vec
                .get(position.chunk)
                .and_then(|chunk| chunk.get(position.pos))
                .and_then(|item| item.as_ref())
        }

        pub(super) fn get_mut(&mut self, position: &Position) -> Option<&mut T> {
            self.vec
                .get_mut(position.chunk)
                .and_then(|chunk| chunk.get_mut(position.pos))
                .and_then(|item| item.as_mut())
        }

        fn get_mut_slot(&mut self, position: &Position) -> &mut Option<T> {
            &mut self.vec[position.chunk][position.pos]
        }

        pub(super) fn advance(&self, position: Position, relative: i64) -> Position {
            // This *isn't* the modulus for the relocation, just for advancing
            // the position for other logic.
            let relative = relative.rem_euclid(self.len as i64) as usize;

            if relative == 0 {
                return position;
            }

            std::iter::successors(Some(position), |pos| {
                Some(Position { ..*pos }.wrapping_succ(CHUNK_SIZE, self.vec.len()))
            })
            // When we start at a position that has been relocated, we need to
            // skip it or we get off-by-one errors.
            .skip(1)
            .filter(|pos| self.get(pos).is_some())
            .nth(relative - 1)
            .expect("non-empty relocation vec")
        }

        pub(super) fn relocate(&mut self, position: Position, relative: i64) -> Position {
            // The actual modulus for relocation is here.
            let mut target = self.advance(position, relative.rem_euclid(self.len as i64 - 1));

            // Special case: Stop 0 from shifting forward one position for no reason.
            if target == position {
                return target;
            }

            target = target.wrapping_succ(CHUNK_SIZE, self.vec.len());

            let mut item = Some(self.get_mut_slot(&position).take().expect("valid position"));
            let mut pos = target;

            while let Some(v) = item {
                item = self.get_mut_slot(&pos).replace(v);
                pos = pos.wrapping_succ(CHUNK_SIZE, self.vec.len());
            }

            target
        }

        #[allow(dead_code)]
        pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
            self.vec
                .iter()
                .flat_map(|chunk| chunk.iter())
                .filter_map(|item| item.as_ref())
        }
    }

    impl<T, const CHUNK_SIZE: usize> FromIterator<T> for RelocationVec<T, CHUNK_SIZE> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let iter = iter.into_iter();
            let (min, _max) = iter.size_hint();

            let mut vec: Vec<[Option<T>; CHUNK_SIZE]> =
                std::iter::repeat_with(|| [(); CHUNK_SIZE].map(|_| None))
                    .take(2 * min / CHUNK_SIZE + 1)
                    .collect::<Vec<_>>();

            let (mut chunk, mut pos) = (0, 0);
            let mut len = 0;
            for (i, item) in iter.into_iter().enumerate() {
                vec[chunk][pos] = Some(item);
                pos += 1;
                if pos * 2 >= CHUNK_SIZE {
                    pos = 0;
                    chunk += 1;
                }

                len = i + 1;
            }

            Self { vec, len }
        }
    }

    // IntoIterator
    impl<T, const CHUNK_SIZE: usize> IntoIterator for RelocationVec<T, CHUNK_SIZE> {
        type Item = T;
        type IntoIter = IntoIter<T, CHUNK_SIZE>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter {
                iter: self.vec.into_iter().flatten(),
            }
        }
    }

    pub(super) struct IntoIter<T, const CHUNK_SIZE: usize> {
        iter: std::iter::Flatten<<Vec<[Option<T>; CHUNK_SIZE]> as IntoIterator>::IntoIter>,
    }

    impl<T, const CHUNK_SIZE: usize> Iterator for IntoIter<T, CHUNK_SIZE> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                match self.iter.next() {
                    Some(Some(item)) => return Some(item),
                    Some(None) => continue,
                    None => return None,
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_relocation_vec() {
            let numbers = (0..100).collect::<Vec<_>>();

            let vec = numbers.iter().copied().collect::<RelocationVec<i32>>();
            assert_eq!(vec.vec.len(), 7);
            let shape = [16, 16, 16, 16, 16, 16, 4];
            let iter = vec.vec.iter().zip(shape);

            let recovered = iter
                .clone()
                .flat_map(|(chunk, len)| chunk.iter().take(len).map(|x| x.unwrap()))
                .collect::<Vec<_>>();

            assert_eq!(recovered, numbers);

            let tail = iter
                .map(|(chunk, len)| chunk.iter().skip(len))
                .flatten()
                .collect::<Vec<_>>();

            assert!(tail.iter().all(|x| x.is_none()));

            let collected = vec.into_iter().collect::<Vec<_>>();

            assert_eq!(&collected, &numbers);
        }

        #[test]
        fn test_relocate() {
            let mut vec = [1, 2, -3, 3, -2, 0, 4]
                .into_iter()
                .collect::<RelocationVec<i64, 4>>();

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, -3, 3, -2, 0, 4]
            );

            let pos = vec.start();
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 1);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&1));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![2, 1, -3, 3, -2, 0, 4]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 2);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&2));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, -3, 2, 3, -2, 0, 4]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 1); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, -3);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&-3));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, 3, -2, -3, 0, 4]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 1); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 2); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 3);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&3));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, -2, -3, 0, 3, 4]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, -2);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&-2));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, -3, 0, 3, 4, -2]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 2); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, -3); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 0);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&0));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, -3, 0, 3, 4, -2]
            );

            let pos = vec.advance(pos, -value);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 0); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 3); // Already moved
            let pos = vec.advance(pos, 1);
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 4);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&4));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![1, 2, -3, 4, 0, 3, -2]
            );
        }

        #[test]
        fn test_relocate_2() {
            let mut vec = [10, 20, -30, 40, 50, 60, 70, 80, 90, 100]
                .into_iter()
                .collect::<RelocationVec<i64, 4>>();

            let pos = Position { chunk: 0, pos: 1 };
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, 20);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&20));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![10, -30, 40, 20, 50, 60, 70, 80, 90, 100]
            );

            let pos = Position { chunk: 1, pos: 0 };
            let &value = vec.get(&pos).unwrap();
            assert_eq!(value, -30);

            let pos = vec.relocate(pos, value);
            assert_eq!(vec.get(&pos), Some(&-30));

            assert_eq!(
                vec.iter().copied().collect::<Vec<_>>(),
                vec![10, 40, 20, 50, 60, 70, 80, -30, 90, 100]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix() {
        let mut vec = [
            8, 2, 32, -41, 6, 29, -4, 6, -8, 8, -3, -8, 3, -5, 0, -1, 2, 1, 10, -9,
        ]
        .into_iter()
        .map(|x| (x, false))
        .collect::<RelocationVec<(i64, _), 4>>();

        let mut steps = vec![];

        mix(&mut vec, |vec| {
            steps.push(vec.iter().map(|(x, _)| *x).collect::<Vec<_>>())
        });

        // Test cases lifted from a Reddit comment.
        let expected = vec![
            vec![
                8, 2, 32, -41, 6, 29, -4, 6, -8, 8, -3, -8, 3, -5, 0, -1, 2, 1, 10, -9,
            ],
            vec![
                2, 32, -41, 6, 29, -4, 6, -8, 8, 8, -3, -8, 3, -5, 0, -1, 2, 1, 10, -9,
            ],
            vec![
                32, -41, 2, 6, 29, -4, 6, -8, 8, 8, -3, -8, 3, -5, 0, -1, 2, 1, 10, -9,
            ],
            vec![
                -41, 2, 6, 29, -4, 6, -8, 8, 8, -3, -8, 3, -5, 32, 0, -1, 2, 1, 10, -9,
            ],
            vec![
                2, 6, 29, -4, 6, -8, 8, 8, -3, -8, 3, -5, 32, 0, -1, 2, -41, 1, 10, -9,
            ],
            vec![
                2, 29, -4, 6, -8, 8, 8, 6, -3, -8, 3, -5, 32, 0, -1, 2, -41, 1, 10, -9,
            ],
            vec![
                2, -4, 6, -8, 8, 8, 6, -3, -8, 3, -5, 29, 32, 0, -1, 2, -41, 1, 10, -9,
            ],
            vec![
                2, 6, -8, 8, 8, 6, -3, -8, 3, -5, 29, 32, 0, -1, 2, -41, -4, 1, 10, -9,
            ],
            vec![
                2, -8, 8, 8, 6, -3, -8, 6, 3, -5, 29, 32, 0, -1, 2, -41, -4, 1, 10, -9,
            ],
            vec![
                2, 8, 8, 6, -3, -8, 6, 3, -5, 29, 32, 0, -8, -1, 2, -41, -4, 1, 10, -9,
            ],
            vec![
                2, 8, 6, -3, -8, 6, 3, -5, 29, 32, 8, 0, -8, -1, 2, -41, -4, 1, 10, -9,
            ],
            vec![
                2, 8, 6, -8, 6, 3, -5, 29, 32, 8, 0, -8, -1, 2, -41, -4, 1, 10, -9, -3,
            ],
            vec![
                2, 8, 6, 6, 3, -5, 29, 32, 8, 0, -8, -1, 2, -41, -8, -4, 1, 10, -9, -3,
            ],
            vec![
                2, 8, 6, 6, -5, 29, 32, 3, 8, 0, -8, -1, 2, -41, -8, -4, 1, 10, -9, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 3, 8, 0, -8, -1, 2, -41, -8, -4, 1, 10, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 3, 8, 0, -8, -1, 2, -41, -8, -4, 1, 10, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 3, 8, 0, -1, -8, 2, -41, -8, -4, 1, 10, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 3, 8, 0, -1, -8, -41, -8, 2, -4, 1, 10, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 3, 8, 0, -1, -8, -41, -8, 2, -4, 10, 1, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 10, 3, 8, 0, -1, -8, -41, -8, 2, -4, 1, -9, -5, -3,
            ],
            vec![
                2, 8, 6, 6, 29, 32, 10, 3, -9, 8, 0, -1, -8, -41, -8, 2, -4, 1, -5, -3,
            ],
        ];

        for (i, (step, expected)) in steps.iter().zip(expected).enumerate() {
            assert_eq!(step, &expected, "step {}", i);
        }

        assert_eq!(
            vec.into_iter().map(|(x, _)| x).collect::<Vec<_>>(),
            vec![2, 8, 6, 6, 29, 32, 10, 3, -9, 8, 0, -1, -8, -41, -8, 2, -4, 1, -5, -3]
        );
    }
}
