use std::sync::Arc;

use worth_foundational::{FoundationalBranchReferenceMismatchAxis, FoundationalBranchTarget};

use super::{
    RelationalBranchBasisDenial, RelationalBranchBasisDescriptor,
    RelationalBranchReferenceObservation, RelationalBranchRoot, RelationalBranchVersion,
};
use crate::history::data::CommitId;

pub(crate) fn reject_cross_branch_target_substitution(
    history: &crate::runtime::HistorySubsystem,
    branch_cell: &super::RelationalBranchReferenceCell,
    descriptor: &RelationalBranchBasisDescriptor,
    current_reference: &RelationalBranchReferenceObservation,
) -> Result<(), RelationalBranchBasisDenial> {
    if descriptor.reference() == current_reference {
        return Ok(());
    }
    let FoundationalBranchTarget::Basis(target) = descriptor.reference().target() else {
        return Ok(());
    };
    let inherited_fork_target = branch_cell
        .fork_provenance()
        .is_some_and(|provenance| provenance.target() == descriptor.reference().target());
    let target_belongs_to_other_branch = history
        .commit_artifact(CommitId(target.selected_commit_id()))
        .is_some_and(|artifact| artifact.identity().authoring_branch() != descriptor.branch_id());
    if target_belongs_to_other_branch && !inherited_fork_target {
        return Err(RelationalBranchBasisDenial::WrongImmutableTarget);
    }
    Ok(())
}

pub(crate) fn require_current_descriptor_axes(
    descriptor: &RelationalBranchBasisDescriptor,
    current_reference: &RelationalBranchReferenceObservation,
    current_truth_version: RelationalBranchVersion,
    root: &Arc<RelationalBranchRoot>,
) -> Result<(), RelationalBranchBasisDenial> {
    if descriptor.reference() != current_reference {
        let mismatch = descriptor
            .reference()
            .compare(current_reference)
            .expect_err("different references report at least one axis");
        if mismatch
            .axes()
            .contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
            && descriptor.reference().target() == current_reference.target()
        {
            return Err(RelationalBranchBasisDenial::StaleReferenceGeneration);
        }
        return Err(
            match (descriptor.reference().target(), current_reference.target()) {
                (FoundationalBranchTarget::Empty, FoundationalBranchTarget::Basis(_))
                | (FoundationalBranchTarget::Basis(_), FoundationalBranchTarget::Empty) => {
                    RelationalBranchBasisDenial::EmptyCommittedTargetMismatch
                }
                _ => RelationalBranchBasisDenial::UnavailableRetainedTarget,
            },
        );
    }
    if descriptor.truth_version() != current_truth_version {
        return Err(RelationalBranchBasisDenial::WrongBranchLocalTruthVersion);
    }
    if descriptor.root_identity() != root.id() {
        return Err(RelationalBranchBasisDenial::MixedAxis(
            super::RelationalBranchBasisMismatchAxis::RootIdentity,
        ));
    }
    let schema_commitment = root.schema_authority().authority_digest();
    if descriptor.schema_commitment() != schema_commitment {
        return Err(RelationalBranchBasisDenial::MixedAxis(
            super::RelationalBranchBasisMismatchAxis::SchemaRoot,
        ));
    }
    let visibility = root
        .axes()
        .map(|axes| axes.visibility.digest())
        .unwrap_or([0; 32]);
    if descriptor.visibility_commitment() != visibility {
        return Err(RelationalBranchBasisDenial::MixedAxis(
            super::RelationalBranchBasisMismatchAxis::Visibility,
        ));
    }
    require_root_matches_reference(current_reference, root)
}

pub(crate) fn require_root_matches_reference(
    reference: &RelationalBranchReferenceObservation,
    root: &RelationalBranchRoot,
) -> Result<(), RelationalBranchBasisDenial> {
    match reference.target() {
        FoundationalBranchTarget::Empty if root.descriptor().is_none() && root.id() == 0 => Ok(()),
        FoundationalBranchTarget::Empty => {
            Err(RelationalBranchBasisDenial::EmptyCommittedTargetMismatch)
        }
        FoundationalBranchTarget::Basis(target) => {
            let root_descriptor = root
                .descriptor()
                .ok_or(RelationalBranchBasisDenial::UnavailableRetainedTarget)?;
            if root_descriptor.truth_root() != target.roots().truth_root() {
                return Err(RelationalBranchBasisDenial::MixedAxis(
                    super::RelationalBranchBasisMismatchAxis::TruthRoot,
                ));
            }
            if root_descriptor.schema_root() != target.roots().schema_root() {
                return Err(RelationalBranchBasisDenial::MixedAxis(
                    super::RelationalBranchBasisMismatchAxis::SchemaRoot,
                ));
            }
            if root.commit_id().map(|commit| commit.0) != Some(target.selected_commit_id()) {
                return Err(RelationalBranchBasisDenial::MixedAxis(
                    super::RelationalBranchBasisMismatchAxis::Commit,
                ));
            }
            Ok(())
        }
    }
}
