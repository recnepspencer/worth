use super::WorthQueryRunningDirectRun;
use crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity;
use crate::domain_computation::WorthQueryBoundGraphExecutionReceipt;

pub struct WorthQueryCompletedDirectEvidenceOwner<'a> {
    authority: &'a crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    session: &'a crate::domain_computation::WorthQueryExecutionProviderSession,
    logical_run_identity: &'a str,
    execution_snapshot: WorthQueryBoundExecutionSnapshotIdentity,
    receipt: &'a WorthQueryBoundGraphExecutionReceipt,
}

pub struct WorthQueryCompletedDirectGraphExecution {
    running: WorthQueryRunningDirectRun,
    receipt: WorthQueryBoundGraphExecutionReceipt,
}

impl WorthQueryCompletedDirectGraphExecution {
    pub(super) fn new(
        running: WorthQueryRunningDirectRun,
        receipt: WorthQueryBoundGraphExecutionReceipt,
        _owner: super::WorthQueryDirectGraphCompletionPermit,
    ) -> Self {
        Self { running, receipt }
    }

    pub fn run_identity(&self) -> &str {
        self.running.identity()
    }

    pub fn receipt(&self) -> &WorthQueryBoundGraphExecutionReceipt {
        &self.receipt
    }

    pub fn into_running(self) -> WorthQueryRunningDirectRun {
        self.running
    }

    pub fn bind_convergence_candidate_evidence(
        &self,
        candidate_selection_key: &str,
    ) -> Result<
        crate::domain_computation::WorthQueryConvergenceDomainEvidenceBinding,
        crate::domain_computation::WorthQueryConvergenceDomainEvidenceBindingDenial,
    > {
        let owner = WorthQueryCompletedDirectEvidenceOwner {
            authority: self.running.affinity.provider_plan_operation(),
            session: self.running.affinity.provider_plan_session(),
            logical_run_identity: self.running.logical_run_identity(),
            execution_snapshot: WorthQueryBoundExecutionSnapshotIdentity::capture(
                self.running.execution_snapshot_reference().into(),
            ),
            receipt: &self.receipt,
        };
        self.receipt
            .derive_direct_convergence_evidence(owner, candidate_selection_key)
    }
}

impl WorthQueryCompletedDirectEvidenceOwner<'_> {
    pub fn authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.authority
    }

    pub fn session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.session
    }

    pub fn logical_run_identity(&self) -> &str {
        self.logical_run_identity
    }

    pub fn execution_snapshot(
        &self,
    ) -> &WorthQueryBoundExecutionSnapshotIdentity {
        &self.execution_snapshot
    }

    pub fn receipt(&self) -> &WorthQueryBoundGraphExecutionReceipt {
        self.receipt
    }
}
