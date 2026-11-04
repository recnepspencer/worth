//! Project original reference meaning, never a runtime-qualified representation.
use super::Denial;
use crate::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchCellCheckpoint,
    RelationalBranchReferenceObservation,
};
use serde::Serialize;
use worth_foundational::facade::{
    NativeCheckpointBranchObservation as Observation, NativeCheckpointBranchReference as Reference,
    NativeCheckpointBranchTarget as Target,
};

fn observation<'a>(
    branch: &'a str,
    source: &'a RelationalBranchReferenceObservation,
) -> Observation<'a> {
    let target = match source.target() {
        worth_foundational::FoundationalBranchTarget::Empty => Target::Empty,
        worth_foundational::FoundationalBranchTarget::Basis(target) => Target::Canonical {
            commit: target.selected_commit_id(),
            version: target.version_id(),
            parents: std::borrow::Cow::Borrowed(target.parent_commit_ids()),
            truth_root: *target.roots().truth_root(),
            schema_root: *target.roots().schema_root(),
        },
    };
    Observation::from_recorded(branch, source.generation().get(), target)
}

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
    schema_commitment: [u8; 32],
    visibility_commitment: [u8; 32],
    posture: crate::branch::RelationalBranchBasisPosture,
}
pub(super) fn component(descriptor: &RelationalBranchBasisDescriptor) -> Component<'_> {
    Component {
        descriptor_version: descriptor.descriptor_version(),
        reference: Reference::from_recorded(
            observation(&descriptor.branch_id().0, descriptor.reference()),
            descriptor.truth_version().as_u64(),
        ),
        schema_commitment: descriptor.schema_commitment(),
        visibility_commitment: descriptor.visibility_commitment(),
        posture: descriptor.posture(),
    }
}
