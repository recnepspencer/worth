//! Funded Memory freeze, original-frame verification and image projection.
mod frames;
pub(super) use frames::ReadmittedMemoryOriginals;
mod freeze;
mod projection;
use super::super::{current_metadata::FrozenCurrentCheckpointMetadata, Denial};
use crate::durability::{
    checkpoint_admission::EncodedAdmissionRow,
    source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
};
use crate::history::{RelationalCommitArtifact, RetainedCanonicalIndex};
use std::sync::{Arc, Weak};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest,
};
use worth_proof::AuthorityMarker;

pub(super) struct FrozenMemoryOriginals {
    pub(super) current: FrozenCurrentCheckpointMetadata,
    pub(super) artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    pub(super) symbols: crate::symbols::data::StringInterner,
    pub(super) mirror: crate::runtime::PartitionEdition,
    pub(super) schemas: Option<Arc<ReadmittedSourceSchemaCatalog>>,
    pub(super) transitions: crate::branch::RelationalBranchTransitionSequence,
    pub(super) owner: Weak<crate::runtime::RelationalRuntimeState>,
    pub(super) coordinate: worth_foundational::CanonicalDigestId,
}
pub(super) struct VerifiedMemoryProjection {
    pub(super) originals: ExecutionArray<crate::history::data::PositionedCanonicalCommit>,
    pub(super) current: crate::branch::CheckpointBranchMetadata,
    pub(super) rows: ExecutionArray<EncodedAdmissionRow>,
    pub(super) roots: ExecutionImmutableBytes,
    pub(super) artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    pub(super) owner: Weak<crate::runtime::RelationalRuntimeState>,
    pub(super) canonical:
        ExecutionOrderedMap<crate::history::data::CommitId, RetainedCanonicalIndex>,
}
impl AuthorityMarker for VerifiedMemoryProjection {}
fn stop(cause: crate::durability::data::RelationalRecoveryStop) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(cause)
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    crate::durability::checkpoint_history::capture::array_capture_stop(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    stop(crate::durability::checkpoint_history::capture::map_stop(
        cause,
    ))
}
