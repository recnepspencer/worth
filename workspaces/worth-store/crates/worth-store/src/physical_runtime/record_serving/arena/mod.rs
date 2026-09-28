//! Store-owned placement and reuse of extent arena ranges.

mod allocation;
mod evacuation;
mod free_ranges;
mod geometry;
mod policy;
mod reconstruction;
mod reservation;
pub(in crate::physical_runtime::record_serving) use evacuation::ArenaEvacuationLease;
#[cfg(test)]
mod tests;
pub(in crate::physical_runtime::record_serving) use geometry::qualified_arena_alignment;
pub(in crate::physical_runtime::record_serving) use reconstruction::reconstruct;

pub(in crate::physical_runtime::record_serving) use allocation::ExtentArenaAllocationOwner;
pub use policy::{ArenaEvacuationThreshold, ExtentArenaCapacity, ExtentArenaPolicyDenial};
pub(in crate::physical_runtime::record_serving) use reservation::{
    ArenaReservation, ArenaReservationObligation, SharedArenaAllocationOwner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum ArenaAllocationDenial {
    InvalidGeometry,
    Overlap,
    Capacity,
    RangeBudget,
    StaleReservation,
    EvacuationBusy,
}
