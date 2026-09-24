use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::BackgroundPacingOutcome;

use crate::physical_runtime::instance::{
    PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime,
};
use crate::physical_runtime::record_serving::RecordWorkAdmission;
use crate::physical_runtime::work::{
    CompletedPhysicalWalReclamationAction, PhysicalWalReclamationScope,
    PhysicalWorkAdmissionAuthority,
};
use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalMutationWorkRequest, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkExecution, PhysicalWorkReadiness, PhysicalWorkScheduler,
    PhysicalWorkSettlementEvidence,
};

pub(super) enum PhysicalWalReclamationActionFailure {
    RuntimeReleased,
    SubmissionDenied,
    SubmissionDeferred,
    SubmissionStale,
    SubmissionFailed,
    PreEffect,
    DependencyBlocked,
    SchedulerCapacityUnavailable,
    BackgroundUnavailable,
    SchedulerDemandRejected,
    QueueAdmissionRejected,
    Command,
    MediaDeniedBeforeEffect,
    EffectRequiresInspection,
    StaleOrForeignSettlement,
}

#[derive(Clone)]
pub(super) struct PhysicalWalReclamationWorkPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
}

impl PhysicalWalReclamationWorkPort {
    pub(super) fn new(
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
        }
    }

    pub(super) fn execute(
        &self,
        scope: PhysicalWalReclamationScope,
        foreground_pressure_events: u64,
    ) -> Result<CompletedPhysicalWalReclamationAction, PhysicalWalReclamationActionFailure> {
        let command = self.prepare_command(scope, foreground_pressure_events)?;
        let outcome = self
            .execution
            .execute_physical_work(command)
            .map_err(|_| PhysicalWalReclamationActionFailure::PreEffect)?;
        match outcome.into_settled().into_evidence() {
            PhysicalWorkSettlementEvidence::WalReclamation { physical, .. }
                if physical.checkpoint() == scope.checkpoint()
                    && physical.segment() == scope.segment()
                    && physical.lsn_range() == scope.lsn_range()
                    && physical.byte_count() == scope.byte_count() =>
            {
                Ok(physical)
            }
            PhysicalWorkSettlementEvidence::NoEffect(_) => {
                Err(PhysicalWalReclamationActionFailure::MediaDeniedBeforeEffect)
            }
            PhysicalWorkSettlementEvidence::TerminalFailure(_) => {
                Err(PhysicalWalReclamationActionFailure::EffectRequiresInspection)
            }
            _ => Err(PhysicalWalReclamationActionFailure::StaleOrForeignSettlement),
        }
    }

    fn prepare_command(
        &self,
        scope: PhysicalWalReclamationScope,
        foreground_pressure_events: u64,
    ) -> Result<PhysicalExecutorCommand, PhysicalWalReclamationActionFailure> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWalReclamationActionFailure::RuntimeReleased)?;
        let request = PhysicalMutationWorkRequest::wal_reclamation(
            scope,
            self.record.wal_reclamation_basis(),
            self.record.security(),
        )
        .map_err(|_| PhysicalWalReclamationActionFailure::SubmissionDenied)?;
        let receipt = submit(&runtime, request)?;
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(|_| PhysicalWalReclamationActionFailure::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(|_| PhysicalWalReclamationActionFailure::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(_) => {
                return Err(PhysicalWalReclamationActionFailure::DependencyBlocked)
            }
        };
        let (pacing, backend, policy, capacity) = self
            .scheduler
            .wal_reclamation_background(
                self.record.scheduler_security(),
                scope.byte_count(),
                foreground_pressure_events,
            )
            .map_err(|_| PhysicalWalReclamationActionFailure::SchedulerCapacityUnavailable)?;
        let lease = match require_complete_lease(pacing) {
            Ok(lease) => lease,
            Err(failure) => return Err(failure),
        };
        let demand = PhysicalSchedulerDemand::wal_reclamation_background(ready, lease, capacity)
            .map_err(|_| PhysicalWalReclamationActionFailure::SchedulerDemandRejected)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(|_| PhysicalWalReclamationActionFailure::PreEffect)?;
        let work = PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(|_| PhysicalWalReclamationActionFailure::QueueAdmissionRejected)?;
        PhysicalExecutorCommand::wal_reclamation(work)
            .map_err(|_| PhysicalWalReclamationActionFailure::Command)
    }
}

fn submit(
    runtime: &PhysicalStoreWorkRuntime,
    request: PhysicalMutationWorkRequest,
) -> Result<
    crate::physical_runtime::PhysicalWorkSubmissionReceipt,
    PhysicalWalReclamationActionFailure,
> {
    match runtime
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => Ok(receipt),
        TransitionOutcome::Denied(_) => Err(PhysicalWalReclamationActionFailure::SubmissionDenied),
        TransitionOutcome::Deferred(_) => {
            Err(PhysicalWalReclamationActionFailure::SubmissionDeferred)
        }
        TransitionOutcome::Stale(_) => Err(PhysicalWalReclamationActionFailure::SubmissionStale),
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(_) => Err(PhysicalWalReclamationActionFailure::SubmissionFailed),
    }
}

fn require_complete_lease(
    pacing: BackgroundPacingOutcome,
) -> Result<
    worth_store_io_scheduler::BackgroundIdleCapacityLease,
    PhysicalWalReclamationActionFailure,
> {
    match pacing {
        BackgroundPacingOutcome::AdmittedWithDebt(admitted) => Ok(admitted.into_lease()),
        BackgroundPacingOutcome::Yield(_)
        | BackgroundPacingOutcome::Deferred(_)
        | BackgroundPacingOutcome::Denied(_)
        | BackgroundPacingOutcome::Throttled(_)
        | BackgroundPacingOutcome::Violation(_) => {
            Err(PhysicalWalReclamationActionFailure::BackgroundUnavailable)
        }
    }
}
