use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::MonotonicClock;

/// Memory-only request control. Every retained constraint keeps its clock;
/// independent ancestor budgets remain effective on different timelines.
#[derive(Clone, Debug)]
pub struct MonotonicDeadline(Deadline);

#[derive(Clone, Debug)]
struct ClockDeadline {
    instant: Instant,
    clock: MonotonicClock,
}

#[derive(Clone, Debug)]
enum Deadline {
    At(ClockDeadline),
    Many(Arc<[ClockDeadline]>),
}

impl MonotonicClock {
    pub fn deadline_at(&self, instant: Instant) -> MonotonicDeadline {
        MonotonicDeadline(Deadline::At(ClockDeadline {
            instant,
            clock: self.clone(),
        }))
    }
}

impl MonotonicDeadline {
    fn constraints(&self) -> &[ClockDeadline] {
        match &self.0 {
            Deadline::At(deadline) => std::slice::from_ref(deadline),
            Deadline::Many(deadlines) => deadlines,
        }
    }

    pub fn instant_on(&self, expected: &MonotonicClock) -> Option<Instant> {
        match &self.0 {
            Deadline::At(deadline) if deadline.clock.same_clock(expected) => Some(deadline.instant),
            Deadline::At(_) | Deadline::Many(_) => None,
        }
    }

    /// O(c) observations for c distinct clock owners, with no allocation.
    pub fn is_elapsed(&self) -> bool {
        self.constraints()
            .iter()
            .any(|deadline| deadline.clock.now() >= deadline.instant)
    }

    /// O(c) observations for c distinct clock owners, with no allocation.
    pub fn remaining(&self) -> Duration {
        self.constraints()
            .iter()
            .map(|deadline| {
                deadline
                    .instant
                    .saturating_duration_since(deadline.clock.now())
            })
            .min()
            .expect("a constructed deadline retains at least one clock")
    }

    /// One minimum per clock bounds observation by distinct owners, even after
    /// repeated capture through both active scopes and lease ancestry.
    /// Same-clock scalar capture is O(1); combining independent owners costs
    /// O(c log c) time and O(c) storage for c distinct retained clocks.
    pub fn meet(self, other: Self) -> Self {
        if let (Deadline::At(left), Deadline::At(right)) = (&self.0, &other.0)
            && left.clock.same_clock(&right.clock)
        {
            return if left.instant <= right.instant {
                self
            } else {
                other
            };
        }
        let mut constraints = BTreeMap::<u64, ClockDeadline>::new();
        for deadline in self.constraints().iter().chain(other.constraints()) {
            match constraints.entry(deadline.clock.identity()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(deadline.clone());
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    if deadline.instant < entry.get().instant {
                        entry.insert(deadline.clone());
                    }
                }
            }
        }
        Self(Deadline::Many(constraints.into_values().collect()))
    }

    pub fn meet_optional(left: Option<Self>, right: Option<Self>) -> Option<Self> {
        match (left, right) {
            (Some(left), Some(right)) => Some(left.meet(right)),
            (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
            (None, None) => None,
        }
    }
}

impl PartialEq for MonotonicDeadline {
    fn eq(&self, other: &Self) -> bool {
        let left = self.constraints();
        let right = other.constraints();
        left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                left.instant == right.instant && left.clock.same_clock(&right.clock)
            })
    }
}
impl Eq for MonotonicDeadline {}

#[cfg(test)]
mod tests;
