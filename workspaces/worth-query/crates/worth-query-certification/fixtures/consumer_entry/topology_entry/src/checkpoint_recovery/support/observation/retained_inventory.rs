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

/// Two consecutive complete inventories establish steady custody; subsequent
/// cycles compare every named class, so a growing or alternating owner fails.
pub(in crate::checkpoint_recovery) struct SteadyCycles {
    previous: Option<(Inventory, Inventory)>,
    stable: Option<(Inventory, Inventory)>,
    pub settled_at: Option<u64>,
    following: usize,
}
impl SteadyCycles {
    pub fn new() -> Self {
        Self {
            previous: None,
            stable: None,
            settled_at: None,
            following: 0,
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
        if let Some(stable) = &self.stable {
            assert_eq!(
                &tuple, stable,
                "cycle {cycle}: complete retained classes remain steady"
            );
            self.following += 1;
        } else if self.previous.as_ref() == Some(&tuple) {
            self.stable = Some(tuple.clone());
            self.settled_at = Some(cycle);
        } else {
            assert!(
                cycle < 24,
                "custody never repeats within 24 warm-up cycles: previous={:?}, current={tuple:?}",
                self.previous
            );
        }
        self.previous = Some(tuple);
        self.following >= following
    }
}
