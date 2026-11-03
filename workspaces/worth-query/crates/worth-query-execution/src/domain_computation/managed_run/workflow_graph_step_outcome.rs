use super::super::workflow_graph_chunk::WorthQueryPendingWorkflowGraphChunk;
use super::super::{
    WorthQueryManagedRunTerminalKind, WorthQueryRunningWorkflowRun, WorthQueryWorkflowRunTerminal,
};
use super::WorthQueryActiveWorkflowGraphExecution;
use crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity;
use crate::domain_computation::WorthQueryBoundGraphExecutionReceipt;

#[must_use = "paused graph execution must be advanced, yielded, or explicitly abandoned"]
pub struct WorthQueryPausedWorkflowGraphExecution {
    pub(in crate::domain_computation::managed_run) active: WorthQueryActiveWorkflowGraphExecution,
    pub(in crate::domain_computation::managed_run) safe_point:
        super::super::yield_eligibility::WorthQueryManagedYieldSafePoint,
}

impl WorthQueryPausedWorkflowGraphExecution {
    pub fn run_identity(&self) -> &str {
        self.active.run_identity()
    }

    pub fn advance(
        self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorthQueryWorkflowGraphStepOutcome {
        self.active.advance(execution)
    }

    pub fn yield_run(self) -> super::super::WorthQueryWorkflowYieldOutcome {
        super::super::workflow_yield_transition::yield_workflow_run(self)
    }

    pub fn abandon(self) -> WorthQueryWorkflowGraphStepOutcome {
        self.active.abandon()
    }
}

pub struct WorthQueryCompletedWorkflowGraphExecution {
    running: WorthQueryRunningWorkflowRun,
    receipt: WorthQueryBoundGraphExecutionReceipt,
}

pub(in crate::domain_computation) struct WorthQueryCompletedWorkflowEvidenceOwner<'a> {
    authority: &'a crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    session: &'a crate::domain_computation::WorthQueryExecutionProviderSession,
    logical_run_identity: &'a str,
    stage_identity: &'a str,
    execution_snapshot: WorthQueryBoundExecutionSnapshotIdentity,
    receipt: &'a WorthQueryBoundGraphExecutionReceipt,
}

impl WorthQueryCompletedWorkflowGraphExecution {
    pub(super) fn new(
        running: WorthQueryRunningWorkflowRun,
        receipt: WorthQueryBoundGraphExecutionReceipt,
        _owner: super::WorthQueryWorkflowGraphCompletionPermit,
    ) -> Self {
        Self { running, receipt }
    }

    pub fn run_identity(&self) -> &str {
        self.running.identity()
    }

    pub fn receipt(&self) -> &WorthQueryBoundGraphExecutionReceipt {
        &self.receipt
    }

    pub fn into_running(self) -> WorthQueryRunningWorkflowRun {
        self.running
    }

    pub(crate) fn bind_convergence_candidate_evidence(
        &self,
        stage_identity: &str,
        candidate_selection_key: &str,
    ) -> Result<
        crate::domain_computation::WorthQueryConvergenceDomainEvidenceBinding,
        crate::domain_computation::WorthQueryConvergenceDomainEvidenceBindingDenial,
    > {
        let owner = WorthQueryCompletedWorkflowEvidenceOwner {
            authority: self.running.completed_evidence_authority(),
            session: self.running.completed_evidence_session(),
            logical_run_identity: self.running.logical_run_identity(),
            stage_identity,
            execution_snapshot: WorthQueryBoundExecutionSnapshotIdentity::capture(
                self.running.execution_snapshot_reference().into(),
            ),
            receipt: &self.receipt,
        };
        self.receipt
            .derive_workflow_convergence_evidence(owner, candidate_selection_key)
    }
}

impl WorthQueryCompletedWorkflowEvidenceOwner<'_> {
    pub(in crate::domain_computation) fn authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.authority
    }

    pub(in crate::domain_computation) fn session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.session
    }

    pub(in crate::domain_computation) fn logical_run_identity(&self) -> &str {
        self.logical_run_identity
    }

    pub(in crate::domain_computation) fn stage_identity(&self) -> &str {
        self.stage_identity
    }

    pub(in crate::domain_computation) fn execution_snapshot(
        &self,
    ) -> &WorthQueryBoundExecutionSnapshotIdentity {
        &self.execution_snapshot
    }

    pub(in crate::domain_computation) fn receipt(&self) -> &WorthQueryBoundGraphExecutionReceipt {
        self.receipt
    }
}

pub enum WorthQueryWorkflowGraphStepOutcome {
    /// Refused before observation or provider work; retains the active execution.
    ForeignAdvancementPhase(WorthQueryActiveWorkflowGraphExecution),
    Continue(WorthQueryPausedWorkflowGraphExecution),
    ChunkReady(WorthQueryPendingWorkflowGraphChunk),
    Completed(WorthQueryCompletedWorkflowGraphExecution),
    Cancelled(WorthQueryWorkflowRunTerminal),
    TimedOut(WorthQueryWorkflowRunTerminal),
    Exhausted(WorthQueryWorkflowRunTerminal),
    Degraded(WorthQueryWorkflowRunTerminal),
    Failed(WorthQueryWorkflowRunTerminal),
}

pub(super) fn terminal_outcome(
    terminal: WorthQueryWorkflowRunTerminal,
    kind: WorthQueryManagedRunTerminalKind,
) -> WorthQueryWorkflowGraphStepOutcome {
    match kind {
        WorthQueryManagedRunTerminalKind::Cancelled => {
            WorthQueryWorkflowGraphStepOutcome::Cancelled(terminal)
        }
        WorthQueryManagedRunTerminalKind::TimedOut => {
            WorthQueryWorkflowGraphStepOutcome::TimedOut(terminal)
        }
        WorthQueryManagedRunTerminalKind::Exhausted => {
            WorthQueryWorkflowGraphStepOutcome::Exhausted(terminal)
        }
        WorthQueryManagedRunTerminalKind::Degraded => {
            WorthQueryWorkflowGraphStepOutcome::Degraded(terminal)
        }
        WorthQueryManagedRunTerminalKind::Failed => {
            WorthQueryWorkflowGraphStepOutcome::Failed(terminal)
        }
        WorthQueryManagedRunTerminalKind::Completed => {
            unreachable!("provider completion returns a completion authority")
        }
    }
}
