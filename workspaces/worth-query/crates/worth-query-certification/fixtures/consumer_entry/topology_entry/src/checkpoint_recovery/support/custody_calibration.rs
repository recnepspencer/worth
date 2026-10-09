//! Workload readings choose exercise pressure; layout models certify byte totals.
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
    pub fn required_bytes(&self) -> usize {
        self.required.iter().map(|(_, _, bytes)| bytes).sum()
    }
    pub fn class_bytes(&self, class: &str) -> usize {
        self.required
            .iter()
            .find(|(name, _, _)| *name == class)
            .unwrap()
            .2
    }
    pub fn assert_no_request_or_excess(&self, ample: &Self) {
        assert_eq!(
            self.class_bytes("live_preparation"),
            0,
            "a refused request retains no preparation"
        );
        for (class, _, bytes) in &self.required {
            assert!(
                *bytes <= ample.class_bytes(class),
                "{class}: refused={self:?}, calibration={ample:?}"
            );
        }
        for (class, (_, bytes)) in &self.index {
            assert!(
                *bytes <= ample.index.get(class).map_or(0, |(_, bytes)| *bytes),
                "{class:?}: refused={self:?}, calibration={ample:?}"
            );
        }
        assert!(
            self.lineage <= ample.lineage,
            "lineage: refused={self:?}, calibration={ample:?}"
        );
    }
}

#[derive(Default, Debug)]
pub(in crate::checkpoint_recovery) struct Readings(BTreeMap<&'static str, Inventory>);
impl Readings {
    pub fn record(&mut self, point: &'static str, inventory: Inventory) {
        self.0.insert(point, inventory);
    }
    pub fn record_peak(&mut self, point: &'static str, inventory: Inventory) {
        if self
            .0
            .get(point)
            .is_none_or(|prior| prior.required_bytes() < inventory.required_bytes())
        {
            self.record(point, inventory);
        }
    }
    pub fn at(&self, point: &str) -> &Inventory {
        self.0
            .get(point)
            .expect("the workload recorded its named point")
    }
}

pub(in crate::checkpoint_recovery) fn calibrate(
    workload: impl FnOnce(usize, u64, &mut Readings),
) -> Readings {
    let mut readings = Readings::default();
    workload(8 * 1024 * 1024, 128 * 1024 * 1024, &mut readings);
    readings
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
