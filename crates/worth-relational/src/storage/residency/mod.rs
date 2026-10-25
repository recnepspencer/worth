//! Shared region residency. Only an accepted, counted lease lends read views.
mod cell;
mod lease;
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
