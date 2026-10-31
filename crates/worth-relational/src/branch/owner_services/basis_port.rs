use super::owner_binding::RelationalOwnerServiceBinding;
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchBasisDescriptor,
    RelationalBranchIdentity,
};
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt,
};

/// Cloneable basis service weakly bound to one Relational runtime owner.
#[derive(Debug, Clone)]
pub struct RelationalBranchBasisPort {
    owner: RelationalOwnerServiceBinding,
}

impl RelationalBranchBasisPort {
    /// Begin cleanup without observing another runtime or granting close authority.
    pub fn begin_commit_result_snapshot_cleanup(
        &self,
        result: std::sync::Arc<crate::transactions::data::CommitResult>,
    ) -> super::RelationalCommitResultSnapshotCleanup {
        super::RelationalCommitResultSnapshotCleanup::new(self.owner.clone(), result)
    }
    /// Release only this immutable native-issued result's original registration.
    pub fn release_commit_result_snapshot(
        &self,
        result: &crate::transactions::data::CommitResult,
    ) -> Result<
        super::RelationalCommitResultSnapshotRelease,
        super::RelationalCommitResultSnapshotReleaseDenial,
    > {
        let runtime = self
            .admitted_runtime()
            .map_err(super::RelationalCommitResultSnapshotReleaseDenial::Owner)?;
        super::result_snapshot_release::release(&runtime, result)
    }

    /// Describe the original owner-admitted pinned subject independently of
    /// later branch advancement. This creates no recovery or durability proof.
    pub fn export_recorded_retained_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<
        crate::branch::RecordedRelationalComponentReference,
        crate::branch::RelationalRecordedComponentRefusal,
    > {
        self.admitted_runtime()
            .map_err(crate::branch::RelationalRecordedComponentRefusal::Basis)?
            .export_recorded_retained_component(execution, basis)
    }

    /// Record one exact current component for durable composition. This is
    /// descriptive export only; readmission still belongs to Native recovery.
    pub fn export_recorded_current_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<
        crate::branch::RecordedRelationalComponentReference,
        crate::branch::RelationalRecordedComponentRefusal,
    > {
        self.admitted_runtime()
            .map_err(crate::branch::RelationalRecordedComponentRefusal::Basis)?
            .export_recorded_current_component(execution, basis)
    }

    pub(super) fn new(owner: RelationalOwnerServiceBinding) -> Self {
        Self { owner }
    }

    pub fn observe_branch(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        self.admitted_runtime()?.observe_branch(identity)
    }

    pub fn observe_branch_with_control(
        &self,
        identity: &RelationalBranchIdentity,
        control: &crate::mvcc::RelationalOperationControl,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        self.admitted_runtime()?
            .observe_branch_with_control(identity, control)
    }

    pub fn admit_branch_basis(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.admit_branch_basis(identity)
    }

    pub fn readmit_branch_basis(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.readmit_branch_basis(descriptor)
    }

    pub fn compare_current_exact(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), RelationalBranchBasisDenial> {
        self.admitted_runtime()?.compare_current_exact(basis)
    }

    pub fn readmit_retained_branch_basis(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
        lease: &RelationalBranchRetentionLease,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?
            .readmit_retained_branch_basis(descriptor, lease)
    }

    pub fn retain_component_basis(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.retain_component_basis(basis)
    }

    pub fn release_component_basis(
        &self,
        lease: RelationalBranchRetentionLease,
    ) -> Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial>
    {
        let owner = match self.owner.admitted_runtime() {
            Some(owner) => owner,
            None => {
                return Err(RelationalBranchRetentionReleaseDenial::new(
                    RelationalBranchBasisDenial::OwnerUnavailable,
                    lease,
                ));
            }
        };
        owner.release_component_basis(lease)
    }

    fn admitted_runtime(
        &self,
    ) -> Result<crate::runtime::RelationalRuntime, RelationalBranchBasisDenial> {
        self.owner
            .admitted_runtime()
            .ok_or(RelationalBranchBasisDenial::OwnerUnavailable)
    }
}
