//! Project original reference meaning, never a runtime-qualified representation.
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use worth_relational_contracts::internal_port::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint;
use worth_relational_branch::internal_port::durability::checkpoint_admission::recorded_reference::{observation, recorded_reference};
use serde::Serialize;
use worth_foundational::facade::{
    NativeCheckpointBranchObservation as Observation, NativeCheckpointBranchReference as Reference,
};
use worth_foundational::CanonicalDigestId;

#[derive(Serialize)]
pub(super) struct Predecessor<'a> {
    reference: Reference<'a>,
    fork_provenance: Option<Observation<'a>>,
}

pub(super) fn predecessor(
    cell: &RelationalBranchCellCheckpoint,
) -> Result<Predecessor<'_>, Denial> {
    let fork_provenance = match (&cell.fork_provenance, &cell.fork_source_branch_id) {
        (None, None) => None,
        (Some(source), Some(branch)) => Some(observation(&branch.0, source)),
        (None, Some(_)) | (Some(_), None) => return Err(Denial::OriginalAuthorityUnavailable),
    };
    Ok(Predecessor {
        reference: Reference::from_recorded(
            observation(&cell.branch_id.0, &cell.observation),
            cell.truth_version.as_u64(),
        ),
        fork_provenance,
    })
}

#[derive(Serialize)]
pub(super) struct Component<'a> {
    descriptor_version: u16,
    reference: Reference<'a>,
    schema_commitment: CanonicalDigestId,
    visibility_commitment: CanonicalDigestId,
    posture: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture,
}
pub(super) fn component(descriptor: &RelationalBranchBasisDescriptor) -> Component<'_> {
    Component {
        descriptor_version: descriptor.descriptor_version(),
        reference: recorded_reference(descriptor),
        schema_commitment: CanonicalDigestId::new(descriptor.schema_commitment()),
        visibility_commitment: CanonicalDigestId::new(descriptor.visibility_commitment()),
        posture: descriptor.posture(),
    }
}
