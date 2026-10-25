//! Shared region residency. Only an accepted, counted lease lends read views.
mod cell;
mod kind_authority;
mod lease;
pub(crate) use kind_authority::RelationalRegionKindAuthority;
mod region_capture;
mod region_image;
mod region_load;
#[cfg(test)]
mod tests;
mod views;

pub(crate) use lease::{Full, Head, RelationalResidencyLease};
pub(crate) use region_capture::capture_region_members;
pub(crate) use views::{RelationalRegionFullView, RelationalRegionHeadView};

use cell::ResidentRegion;
pub(crate) use cell::{RelationalColdRegion, RelationalRegionResidency};

mod retention;
pub(crate) use retention::RelationalRegionRetention;

mod checkpoint_capture;
pub(crate) use checkpoint_capture::RelationalCheckpointRegionPublication;

mod demand_failure;
pub(crate) use demand_failure::RelationalResidencyDemandFailure;

mod cell_load_cache;
mod probe;
mod required_symbols;
pub(crate) use probe::{probe_full, PartitionProbe};
