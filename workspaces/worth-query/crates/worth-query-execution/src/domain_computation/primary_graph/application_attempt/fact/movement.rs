//! Whether a fact moved between the basis that read it and a later snapshot.
//!
//! Only this module says a fact did not move: the source comparison of
//! `source_currentness_in`, or the content comparison of a retained fact
//! with the one the partitioned computation's comparator observed for its
//! key. No other code compares observed facts to decide it.

use super::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_contribution::Comparator;

/// Whether a fact moved. No value of it is built outside the fact module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct FactMovement(Movement);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum Movement {
    Unmoved,
    Moved,
}

/// The fact the comparator observed for a retained key. It has no equality
/// of its own: only [`FactMovement::between`] compares it.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct ObservedRetained(
    WorthQueryApplicationObservedFact,
);

impl ObservedRetained {
    /// Only the holder of the comparator's key observes a retained fact.
    pub(in crate::domain_computation::primary_graph) const fn new(
        _: &Comparator,
        observed: WorthQueryApplicationObservedFact,
    ) -> Self {
        Self(observed)
    }
}

impl FactMovement {
    pub(super) const fn from_equal(equal: bool) -> Self {
        Self(if equal {
            Movement::Unmoved
        } else {
            Movement::Moved
        })
    }

    pub(in crate::domain_computation::primary_graph) const fn movement(self) -> Movement {
        self.0
    }

    /// The retained fact against what the comparator observed for its key.
    pub(in crate::domain_computation::primary_graph) fn between(
        retained: &WorthQueryApplicationObservedFact,
        observed: &ObservedRetained,
    ) -> Self {
        Self::from_equal(*retained == observed.0)
    }
}
