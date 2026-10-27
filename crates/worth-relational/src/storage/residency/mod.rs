//! Shared region residency. Only an accepted, counted lease lends read views.
mod cell;
mod content_capture;
pub(crate) use content_capture::RegionContentFailure;
mod kind_authority;
mod kind_commitment;
mod kind_projection;
pub(crate) use kind_commitment::RelationalRegionKindCommitment;
pub(crate) use kind_projection::rebuild_region_history_kind_buckets;
mod current_edit;
mod full_edit;
pub(crate) use current_edit::{ResidentRegionEdit, ResidentRegionLease};
mod lease;
pub(crate) use full_edit::ResidentFullRegionEdit;
pub(crate) use kind_authority::RelationalRegionKindAuthority;
mod head_adjacency;
mod head_arena;
pub(crate) use head_adjacency::{RelationalHeadAdjacencySet, RelationalHeadAdjacencyTable};
mod history_capture;
mod history_fold;
mod history_kind_cache;
mod region_capture;
mod region_image;
mod region_load;
mod region_publication;
mod resident_history;
pub(crate) use region_publication::{PreparedResidentRegion, RegionCaptureBasis};
pub(crate) use resident_history::ResidentHistoryReadmission;
#[cfg(test)]
mod tests;
mod views;
pub(crate) use head_arena::RelationalRecordHeadArenaView;

pub(crate) use lease::{Full, Head, RelationalResidencyLease};
pub(crate) use region_capture::capture_region_members;
pub(crate) use views::{RelationalRegionFullView, RelationalRegionHeadView};

use cell::ResidentRegion;
pub(crate) use cell::{RelationalColdRegion, RelationalRegionResidency};

mod retention;
pub(crate) use retention::RelationalRegionRetention;

mod checkpoint_capture;
mod checkpoint_kind_images;
use checkpoint_capture::RecordedRegionCheckpoint;
pub(crate) use checkpoint_capture::RelationalCheckpointRegionPublication;
pub(crate) use checkpoint_kind_images::CheckpointRegionKindImages;

mod demand_failure;
pub(crate) use demand_failure::RelationalResidencyDemandFailure;

mod cell_load_cache;
mod discovery_policy;
mod head_probe;
mod probe;
pub(crate) use head_probe::probe_head;
pub(crate) use head_probe::{probe_head_sources, PartitionHeadProbe};
mod required_symbols;
#[cfg(test)]
pub(crate) use probe::probe_full_sources;
pub(crate) use probe::{probe_full, PartitionProbe};
mod symbol_admission_identity;
pub(crate) use symbol_admission_identity::RelationalRegionSymbolAdmissionIdentity;
