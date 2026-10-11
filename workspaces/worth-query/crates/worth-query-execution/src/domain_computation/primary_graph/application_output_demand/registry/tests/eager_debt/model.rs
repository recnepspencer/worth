//! Eager value semantics: no production record, walk, note, or drain types.
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Ready,
    Superseded,
    Closed,
    Pending,
    Running,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub occurrence: usize,
    pub product: usize,
    pub generation: u64,
    pub owners: usize,
    pub phase: Phase,
    pub claims: Vec<usize>,
    pub claim_slots: usize,
    pub source: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Custody {
    pub product: usize,
    pub tokens: usize,
    pub completed: bool,
    pub bound: Option<u64>,
    pub consumed: bool,
    pub denied: bool,
    pub retired: bool,
}
#[derive(Clone, Debug, Default)]
pub(super) struct Model {
    pub rows: BTreeMap<usize, Row>,
    pub custody: BTreeMap<usize, Custody>,
}
impl Model {
    pub fn claimed(&self, id: usize) -> usize {
        self.rows
            .values()
            .map(|r| r.claims.iter().filter(|&&k| k == id).count())
            .sum()
    }
    pub fn awaited(&self, id: usize) -> bool {
        let Some(row) = self.rows.get(&id) else {
            return false;
        };
        if self.rows.values().any(|r| {
            r.occurrence == row.occurrence
                && r.generation > row.generation
                && !matches!(r.phase, Phase::Running | Phase::Pending)
        }) {
            return false;
        }
        self.rows.iter().any(|(&key, r)| {
            r.occurrence == row.occurrence
                && r.generation < row.generation
                && r.phase == Phase::Superseded
                && (r.owners != 0 || self.claimed(key) != 0)
        })
    }
    pub fn required(&self, id: usize) -> bool {
        self.rows
            .get(&id)
            .is_some_and(|r| r.owners != 0 || self.claimed(id) != 0)
    }
    pub fn held(&self, id: usize) -> bool {
        self.required(id) || self.awaited(id)
    }
    pub fn drop_owner(&mut self, id: usize) {
        if let Some(row) = self.rows.get_mut(&id) {
            assert!(row.owners != 0);
            row.owners -= 1;
        }
        self.settle();
    }
    pub fn supersede(&mut self, newest: usize) {
        let row = &self.rows[&newest];
        let (occurrence, generation) = (row.occurrence, row.generation);
        for r in self.rows.values_mut() {
            if r.occurrence == occurrence && r.generation < generation {
                r.phase = Phase::Superseded;
                r.claims.clear();
                r.claim_slots = 0;
            }
        }
        self.settle();
    }
    pub fn release_claims(&mut self, id: usize) {
        let row = self.rows.get_mut(&id).unwrap();
        row.claims.clear();
        row.claim_slots = 0;
        self.settle();
    }
    pub fn retire_product(&mut self, occurrence: usize) {
        for c in self
            .custody
            .values_mut()
            .filter(|c| c.product == occurrence)
        {
            c.retired = true;
        }
        for row in self.rows.values_mut().filter(|r| r.product == occurrence) {
            row.phase = Phase::Closed;
            row.claims.clear();
            row.claim_slots = 0;
        }
        self.rows
            .retain(|_, r| r.product != occurrence || r.owners != 0);
        self.settle();
    }
    pub fn bind(&mut self, entry: usize, generation: u64) {
        self.prune();
        let denied = self.custody.iter().any(|(&id, c)| {
            id != entry && !c.retired && !c.denied && c.bound.is_some_and(|g| g > generation)
        });
        for (&id, c) in &mut self.custody {
            if id != entry && c.bound.is_some_and(|g| g < generation) {
                c.denied = true;
            }
        }
        let c = self.custody.get_mut(&entry).unwrap();
        c.bound = Some(generation);
        c.denied = denied;
    }
    pub fn settle(&mut self) {
        loop {
            let mut changed = false;
            let newest: BTreeMap<_, _> = self
                .rows
                .iter()
                .filter(|(_, r)| r.phase == Phase::Ready)
                .fold(
                    BTreeMap::<usize, (u64, usize)>::new(),
                    |mut map, (&id, r)| {
                        let slot = map.entry(r.occurrence).or_insert((r.generation, id));
                        if r.generation > slot.0 {
                            *slot = (r.generation, id);
                        }
                        map
                    },
                );
            let closed: BTreeSet<_> = self
                .rows
                .iter()
                .filter(|(_, r)| r.owners == 0 && r.phase == Phase::Superseded)
                .map(|(&id, _)| id)
                .collect();
            let old = self.rows.clone();
            for r in self
                .rows
                .values_mut()
                .filter(|r| r.owners == 0 && r.phase == Phase::Ready)
            {
                for claim in &mut r.claims {
                    if closed.contains(claim) {
                        let upstream = &old[claim];
                        if let Some(&(generation, successor)) = newest.get(&upstream.occurrence) {
                            if generation > upstream.generation {
                                *claim = successor;
                                changed = true;
                            }
                        }
                    }
                }
                r.claims.sort_unstable();
                r.claims.dedup();
            }
            let retired: Vec<_> = self
                .rows
                .iter()
                .filter(|(&id, r)| {
                    matches!(r.phase, Phase::Superseded | Phase::Closed | Phase::Pending)
                        && !self.held(id)
                })
                .map(|(&id, _)| id)
                .collect();
            changed |= !retired.is_empty();
            for id in retired {
                self.rows.remove(&id);
            }
            if !changed {
                break;
            }
        }
        self.prune();
    }
    pub fn prune(&mut self) {
        let remove: Vec<_> = self
            .custody
            .iter()
            .filter(|(_, c)| {
                c.tokens == 0
                    && (c.retired
                        || (c.completed
                            && c.consumed
                            && c.bound.is_some()
                            && !self.rows.iter().any(|(&id, r)| {
                                r.source
                                    && (r.owners != 0
                                        || self.awaited(id)
                                        || matches!(r.phase, Phase::Running | Phase::Pending))
                            })))
            })
            .map(|(&id, _)| id)
            .collect();
        for id in remove {
            self.custody.remove(&id);
        }
        if !self.custody.contains_key(&1) {
            for r in self.rows.values_mut() {
                r.source = false;
            }
        }
    }
}
