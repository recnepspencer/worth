//! Checkpoint actions lowered through the canonical Store work runtime.

use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::BackgroundPacingOutcome;

use super::yieldpoint::PhysicalCheckpointYieldpointOwner;
use crate::physical_runtime::instance::{
    PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime,
};
use crate::physical_runtime::record_serving::RecordWorkAdmission;
use crate::physical_runtime::work::{
    CompletedPhysicalCheckpointAction, PhysicalCheckpointRecoveryAction,
    PhysicalCheckpointWorkAction, PhysicalCheckpointWorkScope, PhysicalWorkAdmissionAuthority,
};
use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalMutationWorkRequest, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkExecution, PhysicalWorkReadiness, PhysicalWorkScheduler,
    PhysicalWorkSettlementEvidence,
};

pub(in crate::physical_runtime) enum PhysicalCheckpointActionFailure {
    RuntimeReleased,
    SubmissionDenied,
    SubmissionDeferred,
    SubmissionStale,
    SubmissionFailed,
    PreEffect,
    DependencyBlocked,
    SchedulerCapacityUnavailable,
    BackgroundYielded,
    BackgroundDeferred,
    BackgroundDenied,
    BackgroundThrottled,
    BackgroundViolation,
    SchedulerDemandRejected,
    QueueAdmissionRejected,
    Command,
    MediaDeniedBeforeEffect(worth_store_physical_backend::ArtifactTreeFailure),
    EffectRequiresInspection,
    StaleOrForeignSettlement,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalCheckpointWorkPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
    yieldpoints: Arc<PhysicalCheckpointYieldpointOwner>,
    #[cfg(feature = "certification-test-authority")]
    fail_next_capacity: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl PhysicalCheckpointWorkPort {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        generation: crate::physical_runtime::LifecycleGeneration,
        physical: PhysicalWorkAdmissionAuthority,
        scheduler: PhysicalSchedulerAdmissionOwner,
        record: Arc<RecordWorkAdmission>,
    ) -> Self {
        Self {
            runtime: Arc::downgrade(runtime),
            execution: PhysicalStoreWorkRuntime::execution(runtime, generation),
            physical,
            scheduler,
            record,
            yieldpoints: PhysicalCheckpointYieldpointOwner::new(),
            #[cfg(feature = "certification-test-authority")]
            fail_next_capacity: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub(in crate::physical_runtime) fn release_background_selection(&self) {
        self.scheduler.cancel_checkpoint_background_head();
        self.scheduler.cancel_wal_reclamation_background_head();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn fail_next_admission_after_noting_head(&self) {
        self.fail_next_capacity
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub(in crate::physical_runtime) fn yieldpoints(
        &self,
    ) -> Arc<PhysicalCheckpointYieldpointOwner> {
        Arc::clone(&self.yieldpoints)
    }

    pub(in crate::physical_runtime) fn pause_after(
        &self,
        step: super::yieldpoint::PhysicalCheckpointStep,
    ) {
        self.yieldpoints.pause_after_step(step);
    }

    pub(in crate::physical_runtime) fn execute(
        &self,
        checkpoint: worth_store_physical_format::PhysicalCheckpointIdentity,
        action: PhysicalCheckpointWorkAction,
        payload: Option<Box<[u8]>>,
        foreground_pressure_events: u64,
    ) -> Result<CompletedPhysicalCheckpointAction, PhysicalCheckpointActionFailure> {
        let command =
            self.prepare_command(checkpoint, action, payload, foreground_pressure_events)?;
        let outcome = self
            .execution
            .execute_physical_work(command)
            .map_err(|_denial| PhysicalCheckpointActionFailure::PreEffect)?;
        let settled = match outcome.into_settled().into_evidence() {
            PhysicalWorkSettlementEvidence::Checkpoint { physical, .. }
                if physical.action() == PhysicalCheckpointRecoveryAction::from(action) =>
            {
                Ok(physical)
            }
            PhysicalWorkSettlementEvidence::NoEffect(evidence) => Err(
                PhysicalCheckpointActionFailure::MediaDeniedBeforeEffect(evidence.failure()),
            ),
            PhysicalWorkSettlementEvidence::TerminalFailure(_) => {
                Err(PhysicalCheckpointActionFailure::EffectRequiresInspection)
            }
            _ => Err(PhysicalCheckpointActionFailure::StaleOrForeignSettlement),
        }?;
        self.yieldpoints.pause_after(action);
        Ok(settled)
    }

    fn prepare_command(
        &self,
        checkpoint: worth_store_physical_format::PhysicalCheckpointIdentity,
        action: PhysicalCheckpointWorkAction,
        payload: Option<Box<[u8]>>,
        foreground_pressure_events: u64,
    ) -> Result<PhysicalExecutorCommand, PhysicalCheckpointActionFailure> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalCheckpointActionFailure::RuntimeReleased)?;
        let scope = PhysicalCheckpointWorkScope::new(checkpoint, action)
            .expect("checkpoint action constructors admit nonempty write intervals");
        let request = PhysicalMutationWorkRequest::checkpoint_capture(
            scope,
            self.record.checkpoint_capture_basis(),
            self.record.security(),
        )
        .map_err(|_denial| PhysicalCheckpointActionFailure::SubmissionDenied)?;
        let receipt = submit(&runtime, request)?;
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(|_denial| PhysicalCheckpointActionFailure::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(|_denial| PhysicalCheckpointActionFailure::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(_) => {
                return Err(PhysicalCheckpointActionFailure::DependencyBlocked)
            }
        };
        #[cfg(feature = "certification-test-authority")]
        if self
            .fail_next_capacity
            .swap(false, std::sync::atomic::Ordering::AcqRel)
        {
            self.scheduler.note_checkpoint_background_head();
            return Err(PhysicalCheckpointActionFailure::SchedulerCapacityUnavailable);
        }
        let (pacing, backend, policy, capacity) = self
            .scheduler
            .checkpoint_background(
                self.record.scheduler_security(),
                scope.accounted_bytes(),
                foreground_pressure_events,
            )
            .map_err(|_denial| PhysicalCheckpointActionFailure::SchedulerCapacityUnavailable)?;
        let lease = match require_complete_lease(pacing) {
            Ok(lease) => lease,
            Err(failure) => return Err(failure),
        };
        let demand = PhysicalSchedulerDemand::checkpoint_background(ready, lease, capacity)
            .map_err(|_denial| PhysicalCheckpointActionFailure::SchedulerDemandRejected)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(|_denial| PhysicalCheckpointActionFailure::PreEffect)?;
        let work = PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(|_denial| PhysicalCheckpointActionFailure::QueueAdmissionRejected)?;
        PhysicalExecutorCommand::checkpoint(work, payload)
            .map_err(|_denial| PhysicalCheckpointActionFailure::Command)
    }
}

fn submit(
    runtime: &PhysicalStoreWorkRuntime,
    request: PhysicalMutationWorkRequest,
) -> Result<crate::physical_runtime::PhysicalWorkSubmissionReceipt, PhysicalCheckpointActionFailure>
{
    match runtime
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => Ok(receipt),
        TransitionOutcome::Denied(_) => Err(PhysicalCheckpointActionFailure::SubmissionDenied),
        TransitionOutcome::Deferred(_) => Err(PhysicalCheckpointActionFailure::SubmissionDeferred),
        TransitionOutcome::Stale(_) => Err(PhysicalCheckpointActionFailure::SubmissionStale),
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(_) => Err(PhysicalCheckpointActionFailure::SubmissionFailed),
    }
}

fn require_complete_lease(
    pacing: BackgroundPacingOutcome,
) -> Result<worth_store_io_scheduler::BackgroundIdleCapacityLease, PhysicalCheckpointActionFailure>
{
    match pacing {
        BackgroundPacingOutcome::AdmittedWithDebt(admitted) => Ok(admitted.into_lease()),
        BackgroundPacingOutcome::Yield(_) => {
            Err(PhysicalCheckpointActionFailure::BackgroundYielded)
        }
        BackgroundPacingOutcome::Deferred(_) => {
            Err(PhysicalCheckpointActionFailure::BackgroundDeferred)
        }
        BackgroundPacingOutcome::Denied(_) => {
            Err(PhysicalCheckpointActionFailure::BackgroundDenied)
        }
        BackgroundPacingOutcome::Throttled(_) => {
            Err(PhysicalCheckpointActionFailure::BackgroundThrottled)
        }
        BackgroundPacingOutcome::Violation(_) => {
            Err(PhysicalCheckpointActionFailure::BackgroundViolation)
        }
    }
}
