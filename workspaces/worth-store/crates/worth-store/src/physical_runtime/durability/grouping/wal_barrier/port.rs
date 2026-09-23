use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;

use super::{
    IndeterminatePhysicalWalGroupBarrier, PhysicalWalGroupBarrierDeclaration,
    PhysicalWalGroupBarrierFailureCause, PhysicalWalGroupBarrierOutcome,
    PhysicalWalGroupBarrierSettlement,
};
use crate::physical_runtime::work::PhysicalWorkAdmissionAuthority;
use crate::physical_runtime::{
    instance::{
        PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime, RecordSchedulerReservationDenial,
    },
    record_serving::RecordWorkAdmission,
    PhysicalDurabilityObservation, PhysicalExecutorCommand, PhysicalMutationWorkRequest,
    PhysicalSchedulerDemand, PhysicalWorkAdmission, PhysicalWorkExecution, PhysicalWorkReadiness,
    PhysicalWorkScheduler, PhysicalWorkSettlementEvidence, SealedPhysicalDurabilityGroupMembers,
    WalDurablePhysicalMutationMembers,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalWalGroupBarrierPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
    durability: PhysicalDurabilityObservation,
    wal: crate::physical_runtime::durability::PhysicalWalRuntimeOwner,
}

impl PhysicalWalGroupBarrierPort {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        generation: crate::physical_runtime::LifecycleGeneration,
        physical: PhysicalWorkAdmissionAuthority,
        scheduler: PhysicalSchedulerAdmissionOwner,
        record: Arc<RecordWorkAdmission>,
        durability: PhysicalDurabilityObservation,
        wal: crate::physical_runtime::durability::PhysicalWalRuntimeOwner,
    ) -> Self {
        Self {
            runtime: Arc::downgrade(runtime),
            execution: PhysicalStoreWorkRuntime::execution(runtime, generation),
            physical,
            scheduler,
            record,
            durability,
            wal,
        }
    }

    pub(in crate::physical_runtime) fn synchronize_appended_group(
        &self,
        appended: SealedPhysicalDurabilityGroupMembers,
    ) -> PhysicalWalGroupBarrierOutcome {
        let declaration = match PhysicalWalGroupBarrierDeclaration::for_appended_group(
            &appended,
            self.durability,
        ) {
            Ok(declaration) => declaration,
            Err(denial) => {
                return PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
                    appended,
                    cause: PhysicalWalGroupBarrierFailureCause::Declaration(denial),
                }
            }
        };
        match self.prepare_command(&declaration) {
            Ok(command) => self.execute(appended, declaration, command),
            Err(cause) => PhysicalWalGroupBarrierOutcome::BarrierNotStarted { appended, cause },
        }
    }

    fn prepare_command(
        &self,
        declaration: &PhysicalWalGroupBarrierDeclaration,
    ) -> Result<PhysicalExecutorCommand, PhysicalWalGroupBarrierFailureCause> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWalGroupBarrierFailureCause::RuntimeReleased)?;
        let request = PhysicalMutationWorkRequest::wal_durability_barrier(
            declaration.scope(),
            self.record.wal_barrier_basis(),
            self.record.security(),
        )
        .map_err(PhysicalWalGroupBarrierFailureCause::SubmissionDenied)?;
        let receipt = match runtime
            .submission
            .mutation_submission()
            .submit(request)
            .into_raw()
        {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(denial) => {
                return Err(PhysicalWalGroupBarrierFailureCause::SubmissionDenied(
                    denial,
                ))
            }
            TransitionOutcome::Deferred(deferred) => {
                return Err(PhysicalWalGroupBarrierFailureCause::SubmissionDeferred(
                    deferred,
                ))
            }
            TransitionOutcome::Stale(stale) => {
                return Err(PhysicalWalGroupBarrierFailureCause::SubmissionStale(stale))
            }
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failure) => {
                return Err(PhysicalWalGroupBarrierFailureCause::SubmissionFailed(
                    failure,
                ))
            }
        };
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(PhysicalWalGroupBarrierFailureCause::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(PhysicalWalGroupBarrierFailureCause::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(blocked) => {
                return Err(PhysicalWalGroupBarrierFailureCause::DependencyBlocked {
                    class: blocked.class(),
                    condition: blocked.condition(),
                })
            }
        };
        let (reservation, backend) = self
            .scheduler
            .wal_durability_barrier(self.record.scheduler_security())
            .map_err(|denial: RecordSchedulerReservationDenial| match denial {
                RecordSchedulerReservationDenial::Admission(denial) => {
                    PhysicalWalGroupBarrierFailureCause::SchedulerReservationDenied(denial)
                }
                RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                    PhysicalWalGroupBarrierFailureCause::Scheduler(
                        crate::physical_runtime::PhysicalSchedulerDenial::OwedBackgroundTurn,
                    )
                }
            })?;
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(PhysicalWalGroupBarrierFailureCause::Scheduler)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(PhysicalWalGroupBarrierFailureCause::PreEffect)?;
        let policy =
            crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
        let work = PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(PhysicalWalGroupBarrierFailureCause::Scheduler)?;
        PhysicalExecutorCommand::wal_barrier(
            work,
            declaration.artifact().clone(),
            declaration.binding_digest(),
        )
        .map_err(PhysicalWalGroupBarrierFailureCause::Command)
    }

    fn execute(
        &self,
        appended: SealedPhysicalDurabilityGroupMembers,
        declaration: PhysicalWalGroupBarrierDeclaration,
        command: PhysicalExecutorCommand,
    ) -> PhysicalWalGroupBarrierOutcome {
        let expected_work = command.identity();
        let Some((_expected_scope, expected_binding_digest)) =
            command.wal_barrier_completion_binding()
        else {
            return indeterminate(appended);
        };
        let outcome = match self.execution.execute_physical_work(command) {
            Ok(outcome) => outcome,
            Err(cause) => {
                return PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
                    appended,
                    cause: PhysicalWalGroupBarrierFailureCause::PreEffect(cause),
                }
            }
        };
        let settled = outcome.into_settled();
        let work = settled.intent().identity();
        match settled.into_evidence() {
            PhysicalWorkSettlementEvidence::WalBarrier {
                physical,
                scheduler,
            } => {
                let Some(settlement) = PhysicalWalGroupBarrierSettlement::bind_completed(
                    work,
                    &physical,
                    &scheduler,
                    &declaration,
                    expected_work,
                    expected_binding_digest,
                ) else {
                    return indeterminate(appended);
                };
                if !self.wal.record_durable_barrier(
                    declaration.scope().lsn_start(),
                    declaration.scope().lsn_end_exclusive(),
                ) {
                    return indeterminate(appended);
                }
                match WalDurablePhysicalMutationMembers::derive(appended, settlement) {
                    Ok(durable) => PhysicalWalGroupBarrierOutcome::Durable(durable),
                    Err(appended) => {
                        self.wal.seal_for_inspection();
                        indeterminate(appended)
                    }
                }
            }
            PhysicalWorkSettlementEvidence::NoEffect(evidence) => {
                PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
                    appended,
                    cause: PhysicalWalGroupBarrierFailureCause::MediaDeniedBeforeEffect(
                        evidence.failure(),
                    ),
                }
            }
            _ => indeterminate(appended),
        }
    }
}

fn indeterminate(appended: SealedPhysicalDurabilityGroupMembers) -> PhysicalWalGroupBarrierOutcome {
    PhysicalWalGroupBarrierOutcome::Indeterminate(IndeterminatePhysicalWalGroupBarrier::new(
        appended,
    ))
}
