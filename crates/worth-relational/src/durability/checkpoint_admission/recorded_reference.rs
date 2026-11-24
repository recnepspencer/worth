//! Project exact Branch reference coordinates into their recorded grammar.
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use worth_foundational::facade::{
    NativeCheckpointBranchObservation as Observation, NativeCheckpointBranchReference as Reference,
    NativeCheckpointBranchTarget as Target,
};

pub(crate) fn observation<'a>(
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

pub(crate) fn recorded_reference(descriptor: &RelationalBranchBasisDescriptor) -> Reference<'_> {
    Reference::from_recorded(
        observation(&descriptor.branch_id().0, descriptor.reference()),
        descriptor.truth_version().as_u64(),
    )
}
