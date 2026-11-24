//! Funded Memory freeze, original-frame verification and image projection.
use worth_relational_contracts::internal_port as contracts;
pub mod frames;
pub use frames::ReadmittedMemoryOriginals;

pub mod projection;
use super::super::current_metadata::FrozenCurrentCheckpointMetadata;
use crate::durability::{
    checkpoint_admission::EncodedAdmissionRow,
    source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
};
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest,
};
use worth_proof::AuthorityMarker;
use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact;
use worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex;

pub struct FrozenMemoryOriginals {
    pub current: FrozenCurrentCheckpointMetadata,
    pub indexes: crate::durability::derived_index_artifacts::FrozenIndexCheckpoint,
    pub symbol_frontier: u32,
    pub artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    pub symbols: contracts::symbols::data::string_interner::StringInterner,
    pub mirror:
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition,
    pub schemas: Option<Arc<ReadmittedSourceSchemaCatalog>>,
    pub transitions: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    pub owner:
        worth_relational_branch::internal_port::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin,
    pub coordinate: worth_foundational::CanonicalDigestId,
}
pub struct VerifiedMemoryProjection {
    pub(super) originals: ExecutionArray<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    pub(super) current: worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata,
    pub(super) rows: ExecutionArray<EncodedAdmissionRow>,
    pub(super) roots: ExecutionImmutableBytes,
    pub(super) artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    pub(super) owner:
        worth_relational_branch::internal_port::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin,
    pub(super) canonical:
        ExecutionOrderedMap<contracts::history::data::coordinates::CommitId, RetainedCanonicalIndex>,
}
impl AuthorityMarker for VerifiedMemoryProjection {}
pub fn stop(cause: contracts::durability::data::recovery_stop::RelationalRecoveryStop) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(cause)
}
pub fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    stop(contracts::durability::data::recovery_stop::map_stop(cause))
}
