//! Borrow actual descriptor axes; copy their canonical DATA once after funding.
use super::*;
use crate::branch::basis_axis_validation::{
    require_current_descriptor_axes, require_root_matches_reference,
};
use crate::branch::canonical_runtime_frame::CANONICAL_RUNTIME_INSTANCE_ID;
use crate::branch::{
    RelationalBranchCellCheckpoint, RelationalBranchLifecyclePosture,
    RelationalBranchRootDescriptor, RelationalBranchTarget,
};
use std::cmp::Ordering;
use worth_foundational::FoundationalBranchTarget;

#[derive(Clone, Copy)]
pub(crate) struct CheckpointComponentDescriptorView<'a> {
    branch: &'a BranchId,
    reference: &'a RelationalBranchReferenceObservation,
    truth: RelationalBranchVersion,
    schema: [u8; 32],
    visibility: [u8; 32],
    posture: RelationalBranchBasisPosture,
}
impl<'a> CheckpointComponentDescriptorView<'a> {
    pub(crate) fn current(
        cell: &'a RelationalBranchCellCheckpoint,
        posture: RelationalBranchLifecyclePosture,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::super::RelationalBranchBasisDenial> {
        require_root_matches_reference(&cell.observation, root)?;
        Ok(Self {
            branch: &cell.branch_id,
            reference: &cell.observation,
            truth: cell.truth_version,
            schema: root.schema_authority().authority_digest(),
            visibility: root
                .axes()
                .map(|axes| axes.visibility.digest())
                .unwrap_or([0; 32]),
            posture: match posture {
                RelationalBranchLifecyclePosture::Live => RelationalBranchBasisPosture::Live,
                RelationalBranchLifecyclePosture::Archived => {
                    RelationalBranchBasisPosture::Archived
                }
                RelationalBranchLifecyclePosture::Deleting => {
                    RelationalBranchBasisPosture::Deleting
                }
            },
        })
    }
    pub(crate) fn retained(
        basis: &'a AdmittedRelationalBranchBasis,
    ) -> Result<Self, super::super::RelationalBranchBasisDenial> {
        let descriptor = basis.descriptor();
        if descriptor.descriptor_version() != RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION
            || descriptor.runtime_instance_id() != basis.identity().runtime_instance_id()
            || descriptor.branch_id() != basis.identity().branch_id()
        {
            return Err(super::super::RelationalBranchBasisDenial::WrongImmutableTarget);
        }
        require_current_descriptor_axes(
            descriptor,
            descriptor.reference(),
            descriptor.truth_version(),
            &basis.inner.root,
        )?;
        Ok(Self {
            branch: descriptor.branch_id(),
            reference: descriptor.reference(),
            truth: descriptor.truth_version(),
            schema: descriptor.schema_commitment(),
            visibility: descriptor.visibility_commitment(),
            posture: descriptor.posture(),
        })
    }
    pub(crate) fn allocation_bytes(&self) -> Option<usize> {
        let parents = self
            .reference
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let rendered = "relational//"
            .len()
            .checked_add(10)?
            .checked_add(self.branch.0.len())?;
        self.branch
            .0
            .len()
            .checked_add(rendered)?
            .checked_add(std::alloc::Layout::array::<u64>(parents).ok()?.size())
    }
    /// Pure owner-known copy. The enclosing concrete DATA owner admits the
    /// exact roster and this method's backing quote BEFORE any call.
    pub(crate) fn copy_canonical(
        &self,
    ) -> Result<
        RelationalBranchBasisDescriptor,
        super::super::RelationalBranchObservationConstructionDenial,
    > {
        let target = match self.reference.target() {
            FoundationalBranchTarget::Empty => FoundationalBranchTarget::Empty,
            FoundationalBranchTarget::Basis(target) => {
                FoundationalBranchTarget::Basis(RelationalBranchTarget::new(
                    CANONICAL_RUNTIME_INSTANCE_ID,
                    target.selected_commit_id(),
                    target.version_id(),
                    target.parent_commit_ids().to_vec(),
                    RelationalBranchRootDescriptor::new(
                        *target.roots().truth_root(),
                        *target.roots().schema_root(),
                    ),
                ))
            }
        };
        let reference = super::super::relational_branch_observation(
            CANONICAL_RUNTIME_INSTANCE_ID,
            &self.branch.0,
            target,
            self.reference.generation(),
        )?;
        Ok(RelationalBranchBasisDescriptor::with_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
                branch_id: self.branch.clone(),
                reference,
                truth_version: self.truth,
                root_identity: 0,
                schema_commitment: self.schema,
                visibility_commitment: self.visibility,
            },
            self.posture,
        ))
    }
    /// Exact prior named MessagePack key ordering, without encoded key buffers.
    /// Fixed map field names and canonical runtime/root constants are equal.
    pub(crate) fn canonical_wire_cmp(&self, other: &Self) -> Ordering {
        string_order(&self.branch.0, &other.branch.0)
            .then_with(|| target_order(self.reference.target(), other.reference.target()))
            .then_with(|| {
                self.reference
                    .generation()
                    .cmp(&other.reference.generation())
            })
            .then_with(|| self.truth.cmp(&other.truth))
            .then_with(|| self.schema.cmp(&other.schema))
            .then_with(|| self.visibility.cmp(&other.visibility))
            .then_with(|| string_order(posture(self.posture), posture(other.posture)))
    }
}
fn string_order(left: &str, right: &str) -> Ordering {
    left.len()
        .cmp(&right.len())
        .then_with(|| left.as_bytes().cmp(right.as_bytes()))
}
fn posture(value: RelationalBranchBasisPosture) -> &'static str {
    match value {
        RelationalBranchBasisPosture::Live => "Live",
        RelationalBranchBasisPosture::Archived => "Archived",
        RelationalBranchBasisPosture::Deleting => "Deleting",
    }
}
fn target_order(
    left: &FoundationalBranchTarget<RelationalBranchTarget>,
    right: &FoundationalBranchTarget<RelationalBranchTarget>,
) -> Ordering {
    match (left, right) {
        (FoundationalBranchTarget::Empty, FoundationalBranchTarget::Empty) => Ordering::Equal,
        // Basis is a fixmap(1), Empty a unit string: 0x81 sorts before 0xa5.
        (FoundationalBranchTarget::Basis(_), FoundationalBranchTarget::Empty) => Ordering::Less,
        (FoundationalBranchTarget::Empty, FoundationalBranchTarget::Basis(_)) => Ordering::Greater,
        (FoundationalBranchTarget::Basis(left), FoundationalBranchTarget::Basis(right)) => left
            .selected_commit_id()
            .cmp(&right.selected_commit_id())
            .then_with(|| left.version_id().cmp(&right.version_id()))
            .then_with(|| {
                left.parent_commit_ids()
                    .len()
                    .cmp(&right.parent_commit_ids().len())
            })
            .then_with(|| left.parent_commit_ids().cmp(right.parent_commit_ids()))
            .then_with(|| left.roots().truth_root().cmp(right.roots().truth_root()))
            .then_with(|| left.roots().schema_root().cmp(right.roots().schema_root())),
    }
}

#[cfg(test)]
mod tests;
