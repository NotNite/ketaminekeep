use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Voxel {
    pub id: String,
    pub properties: String,
}

impl Voxel {
    pub fn new(id: String, properties: String) -> Self {
        Self { id, properties }
    }
}

impl PartialEq for Voxel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub struct Box {
    pub min: (usize, usize, usize),
    pub max: (usize, usize, usize),
    pub id: String,
    pub properties: String,
}

impl Box {
    pub fn contains(&self, pos: (usize, usize, usize)) -> bool {
        pos.0 >= self.min.0
            && pos.0 <= self.max.0
            && pos.1 >= self.min.1
            && pos.1 <= self.max.1
            && pos.2 >= self.min.2
            && pos.2 <= self.max.2
    }

    pub fn all_positions(&self) -> Vec<(usize, usize, usize)> {
        let mut positions = Vec::new();
        for x in self.min.0..=self.max.0 {
            for y in self.min.1..=self.max.1 {
                for z in self.min.2..=self.max.2 {
                    positions.push((x, y, z));
                }
            }
        }
        positions
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GreedyDirection {
    X,
    Y,
    Z,
}

pub fn greedy(
    voxels: &HashMap<(usize, usize, usize), Voxel>,
    directions: &[GreedyDirection],
) -> Vec<Box> {
    let mut boxes: Vec<Box> = Vec::new();

    let mut min = (usize::MAX, usize::MAX, usize::MAX);
    let mut max = (usize::MIN, usize::MIN, usize::MIN);
    for (pos, _) in voxels.iter() {
        min.0 = min.0.min(pos.0);
        min.1 = min.1.min(pos.1);
        min.2 = min.2.min(pos.2);
        max.0 = max.0.max(pos.0);
        max.1 = max.1.max(pos.1);
        max.2 = max.2.max(pos.2);
    }

    for x in min.0..=max.0 {
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                let voxel = voxels.get(&(x, y, z));
                if voxel.is_none() {
                    continue;
                }
                let voxel = voxel.unwrap();

                if boxes.par_iter().any(|b| b.contains((x, y, z))) {
                    continue;
                }

                let mut new_box = Box {
                    min: (x, y, z),
                    max: (x, y, z),
                    id: voxel.id.clone(),
                    properties: voxel.properties.clone(),
                };

                // Stop in one of the following conditions:
                // - The next voxel is different (use Eq)
                // - The next voxel is out of bounds
                // - We've reached the max volume
                // - We've hit another box

                for dir in directions {
                    match *dir {
                        GreedyDirection::X => {
                            while new_box.max.0 < max.0 {
                                let next_pos = (new_box.max.0 + 1, new_box.max.1, new_box.max.2);
                                let next_voxel = voxels.get(&next_pos);
                                if next_voxel.is_none() || next_voxel.unwrap() != voxel {
                                    break;
                                }
                                if boxes.par_iter().any(|b| b.contains(next_pos)) {
                                    break;
                                }

                                let mut next_box = new_box.clone();
                                next_box.max.0 += 1;

                                if next_box
                                    .all_positions()
                                    .par_iter()
                                    .any(|pos| voxels.get(pos).map(|v| v != voxel).unwrap_or(true))
                                {
                                    break;
                                }

                                new_box = next_box;
                            }
                        }

                        GreedyDirection::Y => {
                            while new_box.max.1 < max.1 {
                                let next_pos = (new_box.max.0, new_box.max.1 + 1, new_box.max.2);
                                let next_voxel = voxels.get(&next_pos);
                                if next_voxel.is_none() || next_voxel.unwrap() != voxel {
                                    break;
                                }
                                if boxes.par_iter().any(|b| b.contains(next_pos)) {
                                    break;
                                }

                                let mut next_box = new_box.clone();
                                next_box.max.1 += 1;

                                if next_box
                                    .all_positions()
                                    .par_iter()
                                    .any(|pos| voxels.get(pos).map(|v| v != voxel).unwrap_or(true))
                                {
                                    break;
                                }

                                new_box = next_box;
                            }
                        }
                        GreedyDirection::Z => {
                            while new_box.max.2 < max.2 {
                                let next_pos = (new_box.max.0, new_box.max.1, new_box.max.2 + 1);
                                let next_voxel = voxels.get(&next_pos);
                                if next_voxel.is_none() || next_voxel.unwrap() != voxel {
                                    break;
                                }
                                if boxes.par_iter().any(|b| b.contains(next_pos)) {
                                    break;
                                }

                                let mut next_box = new_box.clone();
                                next_box.max.2 += 1;

                                if next_box
                                    .all_positions()
                                    .par_iter()
                                    .any(|pos| voxels.get(pos).map(|v| v != voxel).unwrap_or(true))
                                {
                                    break;
                                }

                                new_box = next_box;
                            }
                        }
                    };
                }

                boxes.push(new_box);
            }
        }
    }

    boxes
}

pub fn flood(voxels: &Vec<(usize, usize, usize)>) -> Vec<Vec<(usize, usize, usize)>> {
    let mut clumps = Vec::new();
    let mut visited = voxels
        .par_iter()
        .map(|pos| (*pos, false))
        .collect::<HashMap<_, _>>();

    for pos in voxels.iter() {
        if *visited.get(pos).unwrap() {
            continue;
        }

        let mut clump = Vec::new();
        let mut stack = vec![*pos];
        while let Some(pos) = stack.pop() {
            if *visited.get(&pos).unwrap() {
                continue;
            }
            visited.insert(pos, true);
            clump.push(pos);

            let neighbors = [
                (pos.0 + 1, pos.1, pos.2),
                (pos.0 - 1, pos.1, pos.2),
                (pos.0, pos.1 + 1, pos.2),
                (pos.0, pos.1 - 1, pos.2),
                (pos.0, pos.1, pos.2 + 1),
                (pos.0, pos.1, pos.2 - 1),
            ];

            for neighbor in neighbors.iter() {
                if *visited.get(neighbor).unwrap_or(&false) {
                    continue;
                }
                if voxels.contains(neighbor) {
                    stack.push(*neighbor);
                }
            }
        }

        clumps.push(clump);
    }

    clumps
}

pub fn best_greedy(voxels: &HashMap<(usize, usize, usize), Voxel>) -> Vec<Box> {
    let mut groups = HashMap::new();
    for (pos, voxel) in voxels.iter() {
        let group = groups
            .entry((voxel.id.clone(), voxel.properties.clone()))
            .or_insert(Vec::new());
        group.push(*pos);
    }

    // Run flood fill on each group to get a list of clumps per group
    let clumps = groups
        .par_iter()
        .flat_map(|(id, group)| {
            let voxels = group.to_vec();
            let clumps = flood(&voxels);
            let mut result = Vec::new();
            for clump in clumps {
                let mut hm = HashMap::new();
                for pos in clump.iter() {
                    hm.insert(*pos, Voxel::new(id.0.clone(), id.1.clone()));
                }
                result.push(hm);
            }

            result
        })
        .collect::<Vec<_>>();

    let directions = [
        [GreedyDirection::X, GreedyDirection::Y, GreedyDirection::Z],
        [GreedyDirection::X, GreedyDirection::Z, GreedyDirection::Y],
        [GreedyDirection::Y, GreedyDirection::X, GreedyDirection::Z],
        [GreedyDirection::Y, GreedyDirection::Z, GreedyDirection::X],
        [GreedyDirection::Z, GreedyDirection::X, GreedyDirection::Y],
        [GreedyDirection::Z, GreedyDirection::Y, GreedyDirection::X],
    ];

    println!("Running greedy on {} clumps", clumps.len());
    // Try each direction on each clump
    clumps
        .par_iter()
        .map(|clump| {
            directions
                .par_iter()
                .map(|directions| {
                    let boxes = greedy(clump, directions);
                    (boxes.len(), boxes)
                })
                .min_by_key(|(volume, _)| *volume)
                .unwrap()
                .1
        })
        .flatten()
        .collect::<Vec<_>>()
}
