use worth_runtime_bridge::facade::BridgeManagedQueueFailureKind;

#[path = "direct_graph_completion.rs"]
mod completion;

pub(in crate::domain_computation) use completion::WorthQueryCompletedDirectEvidenceOwner;
pub use completion::WorthQueryCompletedDirectGraphExecution;

struct WorthQueryDirectGraphCompletionPermit {
    _owner: (),
}

impl WorthQueryDirectGraphCompletionPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

use super::direct_graph_chunk::{
    WorthQueryPendingDirectGraphChunk, WorthQueryPendingDirectGraphQueueState,
};
use super::direct_graph_terminalization::terminal_outcome;
use super::interruption_classification::producer_terminal_kind;
use super::managed_graph_execution::{
    WorthQueryManagedGraphExecution, WorthQueryManagedProviderStep,
};
use super::{
    WorthQueryDirectRunTerminal, WorthQueryManagedRunTerminalKind, WorthQueryRunningDirectRun,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepCompletion;
use crate::domain_computation::{WorthQueryGraphProviderStepReport, WorthQueryGraphReadMaterial};

#[must_use = "active graph execution must be advanced or explicitly abandoned"]
pub struct WorthQueryActiveDirectGraphExecution {
    pub(super) running: WorthQueryRunningDirectRun,
    pub(super) execution: WorthQueryManagedGraphExecution,
}

impl WorthQueryActiveDirectGraphExecution {
    pub(super) fn new(
        running: WorthQueryRunningDirectRun,
        execution: WorthQueryManagedGraphExecution,
    ) -> Self {
        Self { running, execution }
    }

    pub fn run_identity(&self) -> &str {
        self.running.identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.running.logical_run_identity()
    }

    pub fn resource_attempt_identity(&self) -> &str {
        self.running.identity()
    }

    pub fn provider_session_identity(&self) -> &str {
        self.running.provider_session_identity()
    }

    pub fn bridge_basis_identity(&self) -> &str {
        self.running.bridge_basis.identity().as_str()
    }

    pub fn bridge_request_identity(&self) -> &str {
        self.running.bridge_basis.request().digest()
    }

    pub fn provider_call_identity(&self) -> &str {
        self.execution.provider_call_identity()
    }

    pub fn retained_capacity_reservation_count(&self) -> usize {
        self.running.retained_capacity_reservation_count()
    }

    pub fn request_cancellation(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionCancellation,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.running.bridge_basis().request_cancellation(reason)
    }

    pub fn admit_ready_timeout(
        &self,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionTimeout,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.running.bridge_basis().admit_ready_timeout()
    }

    pub fn reject_execution(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionRejectionReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionRejection,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.running.bridge_basis().reject_execution(reason)
    }

    pub fn abandon(self) -> WorthQueryDirectGraphStepOutcome {
        self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed)
    }

    pub fn advance(mut self) -> WorthQueryDirectGraphStepOutcome {
        let before = match self.observe_safe_point() {
            Ok(observation) => observation,
            Err(_) => return self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed),
        };
        let admission = self.execution.admit_provider_step(before);
        self.running
            .provider_work_mut()
            .record_provider_step_admission(admission.counters());
        let admitted = match admission {
            super::provider_step_admission::WorthQueryProviderStepAdmissionOutcome::Admitted(
                admitted,
            ) => admitted,
            super::provider_step_admission::WorthQueryProviderStepAdmissionOutcome::Denied(
                denied,
            ) => return self.interrupted_terminal(denied.terminal()),
        };

        self.running
            .provider_work_mut()
            .record_provider_step_attempt();
        match self.execution.advance_provider(admitted) {
            WorthQueryManagedProviderStep::Failed(evidence) => {
                self.admit_provider_step(evidence.into_report())
            }
            WorthQueryManagedProviderStep::Continue(evidence)
            | WorthQueryManagedProviderStep::Complete(evidence) => {
                self.admit_provider_step(evidence.into_report())
            }
        }
    }

    fn admit_provider_step(
        mut self,
        mut report: WorthQueryGraphProviderStepReport,
    ) -> WorthQueryDirectGraphStepOutcome {
        self.record_report(&mut report);
        if report.completion() == WorthQueryGraphProviderStepCompletion::Failed {
            let _ = self.release_unpublished_projection(&mut report);
            return self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed);
        }
        if let Some(material) = report.take_projection() {
            return self.admit_pending_chunk(report, material);
        }
        match report.completion() {
            WorthQueryGraphProviderStepCompletion::Continue => self.continue_after_safe_point(),
            WorthQueryGraphProviderStepCompletion::Complete => self.finish_completion(&report),
            WorthQueryGraphProviderStepCompletion::Failed => {
                unreachable!("failed provider reports terminalize before output publication")
            }
        }
    }

    fn admit_pending_chunk(
        mut self,
        report: WorthQueryGraphProviderStepReport,
        material: WorthQueryGraphReadMaterial,
    ) -> WorthQueryDirectGraphStepOutcome {
        let width = u64::try_from(material.rows().len()).unwrap_or(u64::MAX);
        let retained_bytes = material.owned_allocation_capacity_bytes();
        if !self.execution.projection_chunk_fits_ceiling() {
            drop(material);
            let _ = self.release_pending_chunk(retained_bytes);
            return self.interrupted_terminal(WorthQueryManagedRunTerminalKind::Exhausted);
        }
        let admission = match self.running.bridge_basis_mut().enqueue_managed_queue(width) {
            Ok(admission) => admission,
            Err(failure) => {
                if !self.release_pending_chunk(retained_bytes) {
                    return self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed);
                }
                return if failure.kind() == BridgeManagedQueueFailureKind::SignalQueueMutationDenied
                {
                    self.interrupted_terminal(WorthQueryManagedRunTerminalKind::Exhausted)
                } else {
                    self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed)
                };
            }
        };
        let (mutation, occupancy) = admission.into_parts();
        self.running
            .provider_work_mut()
            .record_queue_mutation(mutation.counters());
        let queue = WorthQueryPendingDirectGraphQueueState::new(
            occupancy,
            mutation.queue_depth(),
            mutation.queue_capacity(),
        );
        WorthQueryDirectGraphStepOutcome::ChunkReady(WorthQueryPendingDirectGraphChunk::new(
            self,
            report,
            material,
            queue,
            retained_bytes,
        ))
    }

    pub(super) fn finish_completion(
        mut self,
        report: &WorthQueryGraphProviderStepReport,
    ) -> WorthQueryDirectGraphStepOutcome {
        let (receipt, output_retained_bytes, envelope_bytes) = match self
            .execution
            .seal_completion(report)
        {
            Ok(completion) => completion,
            Err(()) => return self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed),
        };
        if !self
            .running
            .provider_work_mut()
            .transfer_output_to_receipt(envelope_bytes, output_retained_bytes)
        {
            return self.settled_terminal(WorthQueryManagedRunTerminalKind::Failed);
        }
        self.running.provider_work_mut().complete_step_call();
        let after = match self.observe_safe_point() {
            Ok(observation) => observation,
            Err(_) => return self.settled_terminal(WorthQueryManagedRunTerminalKind::Failed),
        };
        if let Some(kind) = producer_terminal_kind(&after) {
            return self.settled_terminal(kind);
        }
        let release = self.execution.release_provider_execution();
        let recovery_required = release.evidence().recovery_required();
        let (release_evidence, memory) = release.into_parts();
        self.running
            .provider_work_mut()
            .record_provider_execution_release(&release_evidence);
        self.running
            .provider_work_mut()
            .retain_provider_memory(memory);
        if recovery_required {
            return terminal_outcome(
                self.running
                    .terminal(WorthQueryManagedRunTerminalKind::Failed),
                WorthQueryManagedRunTerminalKind::Failed,
            );
        }
        WorthQueryDirectGraphStepOutcome::Completed(WorthQueryCompletedDirectGraphExecution::new(
            self.running,
            receipt,
            WorthQueryDirectGraphCompletionPermit::mint(),
        ))
    }

    pub(super) fn continue_after_safe_point(mut self) -> WorthQueryDirectGraphStepOutcome {
        let after = match self.observe_safe_point() {
            Ok(observation) => observation,
            Err(_) => return self.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed),
        };
        match producer_terminal_kind(&after) {
            Some(kind) => self.interrupted_terminal(kind),
            None => {
                let safe_point = self.execution.yield_safe_point(after);
                WorthQueryDirectGraphStepOutcome::Continue(WorthQueryPausedDirectGraphExecution {
                    active: self,
                    safe_point,
                })
            }
        }
    }

    fn record_report(&mut self, report: &mut WorthQueryGraphProviderStepReport) {
        self.running.provider_work_mut().admit_step(report);
        self.execution.admit_report(report);
    }

    pub(super) fn observe_safe_point(
        &mut self,
    ) -> Result<
        super::WorthQueryManagedSafePointObservation,
        super::WorthQueryManagedSafePointFailure,
    > {
        let observation = self.running.observe_safe_point()?;
        self.running
            .provider_work_mut()
            .record_safe_point(&observation);
        Ok(observation)
    }

    pub(super) fn release_pending_chunk(&mut self, retained_bytes: usize) -> bool {
        self.execution.release_projection_chunk(retained_bytes)
            && self
                .running
                .provider_work_mut()
                .release_projection_bytes(retained_bytes)
    }

    pub(super) fn retain_pending_chunk(&mut self, material: WorthQueryGraphReadMaterial) -> bool {
        let Some((projection_bytes, additional_bytes)) =
            self.execution.retain_projection_chunk(material)
        else {
            return false;
        };
        self.running
            .provider_work_mut()
            .transfer_projection_to_output(projection_bytes, additional_bytes)
    }

    fn release_unpublished_projection(
        &mut self,
        report: &mut WorthQueryGraphProviderStepReport,
    ) -> bool {
        let Some(material) = report.take_projection() else {
            return true;
        };
        let retained_bytes = material.owned_allocation_capacity_bytes();
        drop(material);
        self.release_pending_chunk(retained_bytes)
    }
}

#[must_use = "paused graph execution must be advanced, yielded, or explicitly abandoned"]
pub struct WorthQueryPausedDirectGraphExecution {
    pub(super) active: WorthQueryActiveDirectGraphExecution,
    pub(super) safe_point: super::yield_eligibility::WorthQueryManagedYieldSafePoint,
}

impl WorthQueryPausedDirectGraphExecution {
    pub fn run_identity(&self) -> &str {
        self.active.run_identity()
    }

    pub fn advance(self) -> WorthQueryDirectGraphStepOutcome {
        self.active.advance()
    }

    pub fn yield_run(self) -> super::WorthQueryDirectYieldOutcome {
        super::direct_yield_transition::yield_direct_run(self)
    }

    pub fn abandon(self) -> WorthQueryDirectGraphStepOutcome {
        self.active.abandon()
    }
}

pub enum WorthQueryDirectGraphStepOutcome {
    Continue(WorthQueryPausedDirectGraphExecution),
    ChunkReady(WorthQueryPendingDirectGraphChunk),
    Completed(WorthQueryCompletedDirectGraphExecution),
    Cancelled(WorthQueryDirectRunTerminal),
    TimedOut(WorthQueryDirectRunTerminal),
    Exhausted(WorthQueryDirectRunTerminal),
    Degraded(WorthQueryDirectRunTerminal),
    Failed(WorthQueryDirectRunTerminal),
}
