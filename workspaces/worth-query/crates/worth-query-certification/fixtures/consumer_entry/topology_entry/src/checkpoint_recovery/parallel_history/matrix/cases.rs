//! One declared product; inapplicable rows carry a name and reason.
pub const UNSEEDED: u64 = u64::MAX;
use super::control::Phase;
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Cell {
    pub family: usize,
    pub workers: usize,
    pub declaration: u64,
    pub dispatch: u64,
    pub fault: usize,
    pub reuse: bool,
}
impl Cell {
    pub fn failure(self, order: &[u64]) -> Option<(u64, Phase)> {
        if self.fault == 0 || self.fault == 10 {
            return None;
        }
        let phase = match (self.fault - 1) / 3 {
            0 => Phase::Prepare,
            1 => Phase::Compute,
            _ => Phase::Complete,
        };
        let rank = match (self.fault - 1) % 3 {
            0 => 0,
            1 => order.len() / 2,
            _ => order.len() - 1,
        };
        Some((order[rank], phase))
    }
}
pub fn table() -> Vec<Cell> {
    let mut cells = Vec::new();
    for family in 0..super::FAMILIES.len() {
        for workers in [1, 2, 5] {
            for declaration in [0, 1] {
                // The fixture cannot read the owned dispatcher's queue witness.
                // Seed perturbation is proved by its owner test, not multiplied here.
                for dispatch in [UNSEEDED] {
                    for fault in 0..10 {
                        for reuse in [false, true] {
                            cells.push(Cell {
                                family,
                                workers,
                                declaration,
                                dispatch,
                                fault,
                                reuse,
                            });
                        }
                    }
                }
            }
        }
    }
    cells.extend(deadlines());
    cells
}
pub fn iteration() -> Vec<Cell> {
    (0..7)
        .flat_map(|family| {
            [1, 2, 5].map(|workers| Cell {
                family,
                workers,
                declaration: 0,
                dispatch: UNSEEDED,
                fault: 0,
                reuse: false,
            })
        })
        .collect()
}

impl Cell {
    pub fn encode(self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.family,
            self.workers,
            self.declaration,
            self.dispatch,
            self.fault,
            u8::from(self.reuse)
        )
    }
    pub fn decode(text: &str) -> Self {
        let v = text
            .split(',')
            .map(|s| s.parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        Self {
            family: usize::try_from(v[0]).unwrap(),
            workers: usize::try_from(v[1]).unwrap(),
            declaration: v[2],
            dispatch: v[3],
            fault: usize::try_from(v[4]).unwrap(),
            reuse: v[5] != 0,
        }
    }
}

pub fn unseeded() -> Vec<Cell> {
    (0..super::FAMILIES.len())
        .flat_map(|family| {
            [1, 2, 5].into_iter().flat_map(move |workers| {
                [0, 2].map(move |fault| Cell {
                    family,
                    workers,
                    declaration: 0,
                    dispatch: UNSEEDED,
                    fault,
                    reuse: false,
                })
            })
        })
        .collect()
}

pub fn deadlines() -> Vec<Cell> {
    (0..super::FAMILIES.len())
        .flat_map(|family| {
            [1, 2, 5].map(|workers| Cell {
                family,
                workers,
                declaration: 0,
                dispatch: UNSEEDED,
                fault: 10,
                reuse: false,
            })
        })
        .collect()
}
