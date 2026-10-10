//! Complete owner inventories certify steady reclamation after warm-up.
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery) struct Inventory {
    pub required: Vec<(&'static str, usize, usize)>,
    pub index: BTreeMap<(&'static str, u32), (usize, u64)>,
    pub lineage: u64,
}
impl Inventory {
    pub fn new(
        required: Vec<(&'static str, usize, usize)>,
        tickets: Vec<(usize, &'static str, u32, u64)>,
        lineage: u64,
    ) -> Self {
        let mut index = BTreeMap::new();
        for (_, file, line, bytes) in tickets {
            let entry = index.entry((file, line)).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += bytes;
        }
        Self {
            required,
            index,
            lineage,
        }
    }
}

/// Complete inventories repeat on the declared one- or two-step schedule;
/// subsequent cycles compare every named class at the same schedule position.
pub(in crate::checkpoint_recovery) struct SteadyCycles {
    previous: [Option<(Inventory, Inventory)>; 2],
    stable: [Option<(Inventory, Inventory)>; 2],
    period: usize,
    pub settled_at: Option<u64>,
    following: usize,
}
impl SteadyCycles {
    pub fn new() -> Self {
        Self {
            previous: std::array::from_fn(|_| None),
            stable: std::array::from_fn(|_| None),
            period: 1,
            settled_at: None,
            following: 0,
        }
    }
    /// Alternating fixture inputs compare each cycle with its own parity.
    pub fn alternating() -> Self {
        Self {
            period: 2,
            ..Self::new()
        }
    }
    pub fn observe(
        &mut self,
        cycle: u64,
        reclaimed: Inventory,
        settled: Inventory,
        following: usize,
    ) -> bool {
        let tuple = (reclaimed, settled);
        let at = cycle as usize % self.period;
        if let Some(stable) = &self.stable[at] {
            assert_eq!(
                &tuple, stable,
                "cycle {cycle}: complete retained classes remain steady"
            );
            if self.settled_at.is_some() {
                self.following += 1;
            }
        } else if self.previous[at].as_ref() == Some(&tuple) {
            self.stable[at] = Some(tuple.clone());
            if self.stable[..self.period].iter().all(Option::is_some) {
                self.settled_at = Some(cycle);
            }
        } else {
            assert!(
                cycle < 24,
                "custody never repeats within 24 warm-up cycles: previous={:?}, current={tuple:?}",
                self.previous[at]
            );
        }
        self.previous[at] = Some(tuple);
        self.following >= following
    }
}
