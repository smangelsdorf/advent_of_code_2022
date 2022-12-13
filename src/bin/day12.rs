use std::collections::HashSet;

struct Heightmap {
    values: Vec<Vec<u8>>,
    costs: Vec<Vec<Option<u64>>>,
    dims: (usize, usize),
}

impl Heightmap {
    fn new(values: Vec<Vec<u8>>) -> Heightmap {
        let costs = values
            .iter()
            .map(|v| v.iter().map(|_| None).collect())
            .collect();

        let dims = (
            values.len(),
            values.iter().map(Vec::len).next().unwrap_or(0),
        );

        let mut map = Heightmap {
            values,
            costs,
            dims,
        };

        for pos in map.find_starts().collect::<Vec<_>>() {
            map.set_cost(pos, 0);
        }

        map
    }

    fn get_height(&self, pos: (usize, usize)) -> u8 {
        let (i, j) = pos;
        match self.values[i][j] {
            b'S' => b'a',
            b'E' => b'z',
            c => c,
        }
    }

    fn get_cost(&self, pos: (usize, usize)) -> Option<u64> {
        let (i, j) = pos;
        self.costs[i][j]
    }

    fn set_cost(&mut self, pos: (usize, usize), cost: u64) {
        let (i, j) = pos;
        self.costs[i][j] = Some(cost);
    }

    fn find_starts(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.find_values(b'S').chain(self.find_values(b'a'))
    }

    fn find_end(&self) -> Option<(usize, usize)> {
        self.find_values(b'E').next()
    }

    fn find_values(&self, value: u8) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.values
            .iter()
            .enumerate()
            .filter_map(move |(i, v)| v.iter().position(|c| *c == value).map(|j| (i, j)))
    }

    fn uncosted_legal_neighbours(
        &self,
        pos: (usize, usize),
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        // Any neighbours that already have a cost were already reached through a shorter path.
        self.neighbours(pos)
            .filter(|next| self.get_cost(*next).is_none())
            .filter(move |next| self.legal(pos, *next))
    }

    fn legal(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        self.get_height(from) + 1 >= self.get_height(to)
    }

    fn neighbours(&self, pos: (usize, usize)) -> impl Iterator<Item = (usize, usize)> + '_ {
        let (i, j) = pos;
        let (i_max, j_max) = self.dims;

        // Unsigned type, so just rely on integer underflow to simplify the bounds check.
        [
            (i.wrapping_sub(1), j),
            (i, j.wrapping_sub(1)),
            (i.wrapping_add(1), j),
            (i, j.wrapping_add(1)),
        ]
        .into_iter()
        .filter(move |&(i, j)| i < i_max && j < j_max)
    }
}

struct Climber {
    positions: HashSet<(usize, usize)>,
    cost: u64,
    target: (usize, usize),
}

impl Climber {
    fn new(map: &Heightmap) -> Climber {
        let positions = map.find_starts().collect();
        let cost = 0;
        let target = map.find_end().expect("destination");

        Climber {
            positions,
            cost,
            target,
        }
    }

    fn step(&mut self, map: &mut Heightmap) {
        let cost = self.cost + 1;
        let positions = self
            .positions
            .drain()
            .flat_map(|pos| map.uncosted_legal_neighbours(pos))
            .collect();

        for &pos in &positions {
            map.set_cost(pos, cost);
        }

        self.cost = cost;
        self.positions = positions;
    }

    fn is_done(&self) -> bool {
        self.positions.contains(&self.target) || self.positions.is_empty()
    }

    fn finished(&self) -> bool {
        self.positions.contains(&self.target)
    }
}

pub fn main() {
    let values = std::io::stdin()
        .lines()
        .map(|r| r.expect("stdin read").into_bytes())
        .collect();

    let mut map = Heightmap::new(values);

    let mut climber = Climber::new(&map);

    while !climber.is_done() {
        climber.step(&mut map);
    }

    if climber.finished() {
        println!("finished, cost: {}", climber.cost);
    } else {
        println!("DNF");
    }
}
