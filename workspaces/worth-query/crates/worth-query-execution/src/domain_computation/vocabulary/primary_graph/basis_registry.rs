//! Moved whole from `domain_computation::primary_graph::application_query::resource_lifecycle::basis_registry`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisReleaseOutcome;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::snapshots::SnapshotId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisIdentity {
    pub(in crate::domain_computation) runtime_instance_id: u64,
    pub(in crate::domain_computation) branch_id: BranchId,
    pub(in crate::domain_computation) snapshot_id: SnapshotId,
    pub(in crate::domain_computation) descriptor: RelationalBranchBasisDescriptor,
    pub(in crate::domain_computation) selection: WorthQueryApplicationBasisSelectionIdentity,
}

impl WorthQueryApplicationBasisIdentity {
    /// Physical Relational clone backing; world selection custody remains shared.
    pub(in crate::domain_computation) fn retained_clone_heap_bytes(&self) -> Option<u64> {
        Self::clone_heap_bytes(&self.branch_id, &self.descriptor)
    }

    /// Price the actual selected descriptor before constructing its Query identity.
    pub(in crate::domain_computation) fn retained_basis_clone_heap_bytes(
        basis: &AdmittedRelationalBranchBasis,
    ) -> Option<u64> {
        Self::clone_heap_bytes(basis.identity().branch_id(), basis.descriptor())
    }

    fn clone_heap_bytes(
        branch_id: &BranchId,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Option<u64> {
        let strings = branch_id
            .0
            .len()
            .checked_add(descriptor.branch_id().0.len())?
            .checked_add(descriptor.reference().branch_id().as_str().len())?;
        let parents = match descriptor.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => 0,
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                std::alloc::Layout::array::<u64>(target.parent_commit_ids().len())
                    .ok()?
                    .size()
            }
        };
        u64::try_from(strings.checked_add(parents)?).ok()
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.descriptor
    }

    pub fn selection(&self) -> &WorthQueryApplicationBasisSelectionIdentity {
        &self.selection
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationBasisReleaseReceipt {
    pub(in crate::domain_computation) identity: WorthQueryApplicationBasisIdentity,
    pub(in crate::domain_computation) outcome: WorthQueryApplicationBasisReleaseOutcome,
}

impl WorthQueryApplicationBasisReleaseReceipt {
    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.identity
    }

    pub const fn released(&self) -> bool {
        self.outcome.released()
    }

    pub const fn custody_released(&self) -> bool {
        self.outcome.custody_released()
    }

    pub const fn outcome(&self) -> WorthQueryApplicationBasisReleaseOutcome {
        self.outcome
    }
}
