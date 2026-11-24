//! Emit the supported cache tuple with each retained root's carried selections.
use super::super::roots::CapturedRoots;
use crate::durability::derived_index_artifacts::{
    CompleteIndexCheckpointView, DerivedIndexCheckpointArtifacts,
};
use crate::durability::log::native_checkpoint_codec::member_frame;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port::indexes::data::basis_checkpoint::IndexBasisCheckpoint;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;

/// A reader admits a selection only for an issued branch and verifies it against
/// that selection's exact root image, so the writer emits it only with both.
pub(in crate::internal_port::durability::authority::checkpoint_complete) fn retained_selection<
    'a,
>(
    roots: &'a CapturedRoots,
    branches: &'a worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata,
) -> impl Fn(&IndexBasisCheckpoint) -> bool + 'a {
    move |basis| {
        roots.retains_basis(basis)
            && branches
                .branch_cells
                .iter()
                .any(|cell| cell.branch_id == basis.branch)
    }
}
pub(in crate::internal_port::durability::authority::checkpoint_complete) fn encode_indexes(
    indexes: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_inputs::OriginalCheckpointIndexes,
    retained: &dyn Fn(&IndexBasisCheckpoint) -> bool,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalDurableMemberLocator, ExecutionImmutableBytes), RelationalRecoveryStop> {
    let checkpoint = CompleteIndexCheckpointView::capture(
        indexes.generations(),
        indexes.bases(),
        retained,
        request,
    )?;
    member_frame::encode_for_request(MemberKind::DerivedIndex, &view(&checkpoint), request)
}
pub(super) fn view<'a>(checkpoint: &'a CompleteIndexCheckpointView<'a>) -> impl Serialize + 'a {
    (
        NoFullCache,
        Some(checkpoint),
        DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
    )
}
/// The tuple's full-cache slot stays present and empty beside a checkpoint payload.
struct NoFullCache;
impl Serialize for NoFullCache {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut row = serializer.serialize_struct("DerivedIndexArtifacts", 1)?;
        row.serialize_field("generations", &[] as &[DerivedIndexGeneration])?;
        row.end()
    }
}
