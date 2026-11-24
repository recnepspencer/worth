//! Shared region residency. Only an accepted, counted lease lends read views.
pub mod source_retirement;
pub use source_retirement::{
    NativeRegionSourceRetirement, NativeRegionSourceRetirementDenial,
    NativeRegionSourceRetirementProgress, NativeSourcePass,
};
pub mod record_directory;
pub use record_directory::{
    DirectoryError as RecordDirectoryError, KindRegionDirectory, PortableRecordDirectory,
    RegionRecordDirectory,
};
pub mod live_counts;
pub use live_counts::LiveRecordCounts;
pub mod cell;
pub mod cold_readmission;
pub mod content_capture;
pub use content_capture::RegionContentFailure;
pub mod kind_admission;
pub mod kind_authority;
pub mod kind_commitment;
pub mod kind_history;
pub mod kind_incremental;
pub mod kind_index;
pub mod kind_projection;
pub use kind_commitment::RelationalRegionKindCommitment;
#[cfg(test)]
pub(crate) use kind_projection::rebuild_region_history_kind_buckets;
pub mod current_edit;
pub mod full_edit;
pub use current_edit::{ResidentRegionEdit, ResidentRegionLease};
pub mod lease;
pub use full_edit::ResidentFullRegionEdit;
pub use kind_authority::RelationalRegionKindAuthority;
pub mod head_adjacency;
pub mod head_arena;
pub use head_adjacency::{RelationalHeadAdjacencySet, RelationalHeadAdjacencyTable};
pub mod history_capture;
pub mod history_capture_bound;
pub mod history_fold;
pub mod history_kind_cache;
pub mod history_presence;
pub mod region_capture;
pub mod region_capture_data;
pub mod region_image;
pub mod region_initialization;
pub mod region_load;
pub mod region_schema;
pub use region_schema::RelationalRegionSchema;
pub mod region_publication;
pub mod resident_history;
pub use region_publication::{PreparedResidentRegion, RegionCaptureBasis};
pub use resident_history::ResidentHistoryReadmission;
#[cfg(test)]
mod tests;
pub mod views;
pub use head_arena::RelationalRecordHeadArenaView;

pub use lease::{Full, Head, RelationalResidencyLease};
pub use views::{RelationalRegionFullView, RelationalRegionHeadView};

pub use cell::ResidentRegion;
pub use cell::{ReconstructedRegionHead, RelationalColdRegion, RelationalRegionResidency};

pub mod retention;
pub use retention::RelationalRegionRetention;

pub mod checkpoint_capture;
pub mod checkpoint_kind_images;
pub use checkpoint_capture::{
    RecordedRegionCheckpoint, RegionCheckpointAcknowledgementDenial,
    RelationalCapturedRegionCheckpoint, RelationalCheckpointRegionPublication,
    RetiredRegionCheckpointBindings,
};
pub use checkpoint_kind_images::CheckpointRegionKindImages;

pub mod demand_failure;
pub use demand_failure::RelationalResidencyDemandFailure;

pub mod cell_load_cache;
pub mod discovery_policy;
pub mod head_probe;
pub mod probe;
pub use head_probe::probe_head;
pub use head_probe::{probe_head_sources, PartitionHeadProbe};
pub mod required_symbols;
#[cfg(test)]
pub use probe::probe_full_sources;
pub use probe::{probe_full, PartitionProbe};
pub mod symbol_admission_identity;
pub mod symbol_changes;
pub use symbol_changes::RegionSymbolChanges;
pub mod symbol_inheritance;
pub use symbol_admission_identity::RelationalRegionSymbolAdmissionIdentity;
pub mod symbol_publication;
pub use symbol_publication::{RegionSymbolPublication, SymbolPublicationKey};

pub use content_capture::restore_failure as content_reconstruction_failure;

pub use checkpoint_capture::verify_checkpoint_directory;
