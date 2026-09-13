use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{TruthBranchIdentity, TruthSnapshotIdentity};

use super::WorthQueryRelationalSourceOwner;

impl WorthQueryRelationalSourceOwner {
    pub fn current_truth_snapshot(
        &self,
        branch: &TruthBranchIdentity,
    ) -> Option<TruthSnapshotIdentity> {
        self.bridge_head
            .lock()
            .expect("Bridge head custody is available")
            .as_ref()
            .filter(|head| head.branch_identity() == branch)
            .map(|head| head.snapshot_identity().clone())
    }

    pub fn bind_current_truth_head(
        &self,
        branch: &BranchId,
    ) -> Result<TruthSnapshotIdentity, RelationalBranchBasisDenial> {
        let basis = self.with_runtime(|runtime| {
            let identity = runtime
                .branch_identity(branch)
                .map_err(|_| RelationalBranchBasisDenial::UnknownBranch(branch.clone()))?;
            runtime.observe_branch(&identity).map(|(_, basis)| basis)
        })?;
        let head = self.source.bind_branch_head_basis_for_bridge(&basis)?;
        let snapshot = head.snapshot_identity().clone();
        *self
            .bridge_head
            .lock()
            .expect("Bridge head custody is available") = Some(head);
        Ok(snapshot)
    }

    pub(crate) fn bind_truth_head_basis_in_runtime(
        &self,
        runtime: &RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<TruthSnapshotIdentity, RelationalBranchBasisDenial> {
        let head = self
            .source
            .bind_branch_head_basis_for_bridge_in_runtime(runtime, basis)?;
        let snapshot = head.snapshot_identity().clone();
        *self
            .bridge_head
            .lock()
            .expect("Bridge head custody is available") = Some(head);
        Ok(snapshot)
    }
}
