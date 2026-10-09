//! Store-owned placement and reuse of extent arena ranges.

mod allocation;
mod evacuation;
mod free_ranges;
mod geometry;
mod policy;
mod reconstruction;
mod released_controls;
mod reservation;
#[cfg(any(test, feature = "certification-test-authority"))]
mod tier_epoch;
pub(in crate::physical_runtime::record_serving) use evacuation::ArenaEvacuationLease;
#[cfg(test)]
mod tests;
pub(in crate::physical_runtime::record_serving) use geometry::qualified_arena_alignment;
pub(in crate::physical_runtime::record_serving) use reconstruction::reconstruct;
pub(in crate::physical_runtime) use released_controls::{
    ReleasedControlArenaPlacement, ReleasedControlArenaReservations,
};

pub(in crate::physical_runtime::record_serving) use allocation::ExtentArenaAllocationOwner;
pub use policy::{ArenaEvacuationThreshold, ExtentArenaCapacity, ExtentArenaPolicyDenial};
pub(in crate::physical_runtime::record_serving) use reservation::{
    ArenaReservation, ArenaReservationObligation, SharedArenaAllocationOwner,
};
#[cfg(any(test, feature = "certification-test-authority"))]
pub(in crate::physical_runtime::record_serving) use tier_epoch::ArenaTierEpochFence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaAllocationDenial {
    InvalidGeometry,
    Overlap,
    Capacity,
    RangeBudget { required: usize, maximum: usize },
    StaleReservation,
    EvacuationBusy,
}
