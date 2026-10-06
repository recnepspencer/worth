//! Incremental topology edits. Pure mutators are offline structural mechanics.
//! Runtime split, removal, and recut use checked methods with a lease and kernel
//! context; their work is already charged when they return. `PartitionWork::charge`
//! is for a bounded pure candidate edit inside an admitted pattern, before the
//! caller publishes that candidate, and must not be called on checked results.

mod bisection;
mod checked_scope;
mod components;
mod keyed;

pub use bisection::{Bisection, BisectionDenial, BisectionQuality, WeightedEdge, WeightedItem};
pub use components::{ComponentDenial, ComponentPartitioner};
pub use keyed::{KeyedDenial, KeyedEditDenial, KeyedItem, KeyedPartitioner};

use worth_foundational::PartitionIdentity;

use crate::{
    authority::LeaseDenial,
    backend::{KernelContext, KernelStop},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionUpdateDenial {
    Components(ComponentDenial),
    Bisection(BisectionDenial),
    Stop(KernelStop),
    Admission(LeaseDenial),
    KernelPanic,
    ResultCapacityExceeded,
}

/// Stable item and originating fact identities are independent of partition identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartitionItemId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceFactId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionRoute {
    pub partition: PartitionIdentity,
    pub source_fact: SourceFactId,
}

/// Structural work in one update. Counts exclude BTree lookup comparisons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PartitionWork {
    pub items_rerouted: u64,
    pub members_visited: u64,
    pub edges_visited: u64,
    pub subtrees_recut: u64,
}

impl PartitionWork {
    /// Charge a bounded pure candidate edit before publication. Checked mutators
    /// already checkpoint internally; charging their result again is an error.
    pub fn charge(self, context: &mut KernelContext<'_, '_>) -> Result<Self, KernelStop> {
        let units = self.units().ok_or(KernelStop::WorkCounterOverflow)?;
        context.checkpoint(units)?;
        Ok(self)
    }

    /// The work units this update is charged, or `None` when they overflow.
    /// A caller that partitions before any pattern is admitted charges them
    /// against its own declared ceiling.
    pub fn units(self) -> Option<u64> {
        self.items_rerouted
            .checked_add(self.members_visited)
            .and_then(|value| value.checked_add(self.edges_visited))
            .and_then(|value| value.checked_add(self.subtrees_recut))
    }
}
