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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        branch: &BranchId,
    ) -> Result<TruthSnapshotIdentity, WorthQuerySourceHeadDenial> {
        let execution = phase.request_for_source(self.runtime_instance_id)?;
        self.bind_current_truth_head_with_request(&execution, branch)
            .map_err(Into::into)
    }

    pub(in crate::domain_computation) fn bind_current_truth_head_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &BranchId,
    ) -> Result<TruthSnapshotIdentity, RelationalBranchBasisDenial> {
        let basis = self.with_runtime(execution, |runtime| {
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

/// An installed head read retains its native or phase custody refusal.
#[derive(Debug)]
pub enum WorthQuerySourceHeadDenial {
    Native(RelationalBranchBasisDenial),
    ForeignPhase(crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase),
}
impl From<RelationalBranchBasisDenial> for WorthQuerySourceHeadDenial {
    fn from(denial: RelationalBranchBasisDenial) -> Self {
        Self::Native(denial)
    }
}
impl From<crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase>
    for WorthQuerySourceHeadDenial
{
    fn from(
        denial: crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    ) -> Self {
        Self::ForeignPhase(denial)
    }
}
