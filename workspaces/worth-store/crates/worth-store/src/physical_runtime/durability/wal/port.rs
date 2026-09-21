use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;
use worth_signal::facade::{AsyncNodeAdmissionClass, AsyncNodeConditionBlockClass};
use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::ArtifactTreeFailure;

use super::PhysicalWalRuntimeOwner;
use crate::physical_runtime::durability::PhysicalDurabilityGroupingRuntimeAuthority;
use crate::physical_runtime::durability::PhysicalMutationIdempotencyRuntimeAuthority;
use crate::physical_runtime::work::PhysicalWorkAdmissionAuthority;
use crate::physical_runtime::{
    instance::{
        PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime, RecordSchedulerReservationDenial,
    },
    record_serving::RecordWorkAdmission,
    PhysicalDurabilityObservation, PhysicalExecutorCommand, PhysicalExecutorCommandDenial,
    PhysicalMutationWorkRequest, PhysicalSchedulerDemand, PhysicalSchedulerDenial,
    PhysicalWalAppendScope, PhysicalWalAppendSettlement, PhysicalWorkAdmission,
    PhysicalWorkExecution, PhysicalWorkPreEffectDenial, PhysicalWorkReadiness,
    PhysicalWorkScheduler, PhysicalWorkSettlementEvidence, WalAppendedPhysicalMutation,
    WalBarrierMember, WalRangeReservedPhysicalMutation,
};

mod group;

pub use group::{
    IndeterminatePhysicalWalGroupAppend, PhysicalWalGroupAppendContinuation,
    PhysicalWalGroupAppendFailureCause, PhysicalWalGroupAppendOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWalAppendFailureCause {
    RuntimeReleased,
    SubmissionDenied(crate::physical_runtime::PhysicalWorkSubmissionDenial),
    SubmissionDeferred(crate::physical_runtime::PhysicalWorkSubmissionDeferred),
    SubmissionStale(crate::physical_runtime::PhysicalWorkSubmissionStale),
    SubmissionFailed(crate::physical_runtime::PhysicalWorkSubmissionFailure),
    PreEffect(PhysicalWorkPreEffectDenial),
    DependencyBlocked {
        class: AsyncNodeAdmissionClass,
        condition: Option<AsyncNodeConditionBlockClass>,
    },
    SchedulerReservationDenied(
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    ),
    Scheduler(PhysicalSchedulerDenial),
    Command(PhysicalExecutorCommandDenial),
    MediaDeniedBeforeEffect(ArtifactTreeFailure),
}

pub(super) enum PhysicalWalGroupMemberAppendOutcome {
    Appended(WalBarrierMember<WalAppendedPhysicalMutation>),
    NotStarted {
        member: WalBarrierMember<WalRangeReservedPhysicalMutation>,
        cause: PhysicalWalAppendFailureCause,
    },
    Indeterminate(WalBarrierMember<WalRangeReservedPhysicalMutation>),
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalWalAppendPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
    owner: PhysicalWalRuntimeOwner,
    grouping: PhysicalDurabilityGroupingRuntimeAuthority,
    idempotency: PhysicalMutationIdempotencyRuntimeAuthority,
    durability: PhysicalDurabilityObservation,
}

impl PhysicalWalAppendPort {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        generation: crate::physical_runtime::LifecycleGeneration,
        physical: PhysicalWorkAdmissionAuthority,
        scheduler: PhysicalSchedulerAdmissionOwner,
        record: Arc<RecordWorkAdmission>,
        owner: PhysicalWalRuntimeOwner,
        grouping: PhysicalDurabilityGroupingRuntimeAuthority,
        idempotency: PhysicalMutationIdempotencyRuntimeAuthority,
        durability: PhysicalDurabilityObservation,
    ) -> Self {
        Self {
            runtime: Arc::downgrade(runtime),
            execution: PhysicalStoreWorkRuntime::execution(runtime, generation),
            physical,
            scheduler,
            record,
            owner,
            grouping,
            idempotency,
            durability,
        }
    }

    pub(super) fn append_group_member(
        &self,
        member: WalBarrierMember<WalRangeReservedPhysicalMutation>,
    ) -> PhysicalWalGroupMemberAppendOutcome {
        let (binding, reserved) = member.into_parts();
        match self.prepare_command(&reserved) {
            Ok(command) => self.execute_group_member(reserved, command),
            Err(cause) => PhysicalWalGroupMemberAppendOutcome::NotStarted {
                member: WalBarrierMember::new(binding, reserved),
                cause,
            },
        }
    }

    pub(in crate::physical_runtime) fn observation(&self) -> super::PhysicalWalObservation {
        self.owner.observation()
    }

    pub(in crate::physical_runtime) fn checkpoint_source_range(
        &self,
    ) -> Option<worth_store_physical_format::CheckpointWalSourceRange> {
        self.owner.checkpoint_source_range()
    }

    pub(in crate::physical_runtime) fn recovery_tail(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryWalTail {
        self.owner.recovery_tail()
    }

    pub(in crate::physical_runtime::durability) fn checkpoint_cutover(
        &self,
    ) -> Option<super::PhysicalWalCheckpointCutover<'_>> {
        self.owner.checkpoint_cutover()
    }

    fn prepare_command(
        &self,
        reserved: &WalRangeReservedPhysicalMutation,
    ) -> Result<PhysicalExecutorCommand, PhysicalWalAppendFailureCause> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWalAppendFailureCause::RuntimeReleased)?;
        let declaration = reserved.declaration();
        let artifact_range = declaration.artifact_range();
        let scope = PhysicalWalAppendScope::new(
            declaration.segment().get(),
            declaration.generation().get(),
            artifact_range.offset(),
            artifact_range.byte_count(),
            declaration.disposition(),
        )
        .expect("reserved WAL declarations carry one valid append scope");
        let request = PhysicalMutationWorkRequest::wal_append(
            scope,
            self.record.wal_append_basis(),
            self.record.security(),
        )
        .map_err(PhysicalWalAppendFailureCause::SubmissionDenied)?;
        let receipt = match runtime
            .submission
            .mutation_submission()
            .submit(request)
            .into_raw()
        {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(denial) => {
                return Err(PhysicalWalAppendFailureCause::SubmissionDenied(denial))
            }
            TransitionOutcome::Deferred(deferred) => {
                return Err(PhysicalWalAppendFailureCause::SubmissionDeferred(deferred))
            }
            TransitionOutcome::Stale(stale) => {
                return Err(PhysicalWalAppendFailureCause::SubmissionStale(stale))
            }
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failure) => {
                return Err(PhysicalWalAppendFailureCause::SubmissionFailed(failure))
            }
        };
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(PhysicalWalAppendFailureCause::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(PhysicalWalAppendFailureCause::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(blocked) => {
                return Err(PhysicalWalAppendFailureCause::DependencyBlocked {
                    class: blocked.class(),
                    condition: blocked.condition(),
                })
            }
        };
        let (reservation, backend) = self
            .scheduler
            .wal_append(
                self.record.scheduler_security(),
                artifact_range.byte_count(),
            )
            .map_err(|denial: RecordSchedulerReservationDenial| match denial {
                RecordSchedulerReservationDenial::Admission(denial) => {
                    PhysicalWalAppendFailureCause::SchedulerReservationDenied(denial)
                }
                RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                    PhysicalWalAppendFailureCause::Scheduler(
                        crate::physical_runtime::PhysicalSchedulerDenial::OwedBackgroundTurn,
                    )
                }
            })?;
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(PhysicalWalAppendFailureCause::Scheduler)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(PhysicalWalAppendFailureCause::PreEffect)?;
        let policy =
            crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
        let work = PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(PhysicalWalAppendFailureCause::Scheduler)?;
        PhysicalExecutorCommand::wal_frame_write(
            work,
            reserved.artifact().clone(),
            reserved.encoded_frame().to_vec().into_boxed_slice(),
        )
        .map_err(PhysicalWalAppendFailureCause::Command)
    }

    fn execute_group_member(
        &self,
        reserved: WalRangeReservedPhysicalMutation,
        command: PhysicalExecutorCommand,
    ) -> PhysicalWalGroupMemberAppendOutcome {
        let binding = reserved.group_binding();
        let expected_work = command.identity();
        let expected_binding = command
            .wal_frame_completion_binding()
            .expect("the WAL port can execute only a typed WAL frame command");
        let outcome = match self.execution.execute_physical_work(command) {
            Ok(outcome) => outcome,
            Err(cause) => {
                return PhysicalWalGroupMemberAppendOutcome::NotStarted {
                    member: WalBarrierMember::new(binding, reserved),
                    cause: PhysicalWalAppendFailureCause::PreEffect(cause),
                };
            }
        };
        let settled = outcome.into_settled();
        let work = settled.intent().identity();
        match settled.into_evidence() {
            PhysicalWorkSettlementEvidence::WalAppend {
                physical,
                scheduler: QueueExecutionOutcome::Executed(_),
            } => self.complete_group_member(
                binding,
                reserved,
                expected_work,
                expected_binding,
                PhysicalWalAppendSettlement::completed_append(work, &physical),
                physical.range().byte_count(),
            ),
            PhysicalWorkSettlementEvidence::WalSegmentCreate {
                physical,
                scheduler: QueueExecutionOutcome::Executed(_),
            } => self.complete_group_member(
                binding,
                reserved,
                expected_work,
                expected_binding,
                PhysicalWalAppendSettlement::completed_segment_create(work, &physical),
                physical.completed_bytes(),
            ),
            PhysicalWorkSettlementEvidence::NoEffect(evidence) => {
                PhysicalWalGroupMemberAppendOutcome::NotStarted {
                    member: WalBarrierMember::new(binding, reserved),
                    cause: PhysicalWalAppendFailureCause::MediaDeniedBeforeEffect(
                        evidence.failure(),
                    ),
                }
            }
            _ => {
                self.owner.seal_for_inspection();
                PhysicalWalGroupMemberAppendOutcome::Indeterminate(WalBarrierMember::new(
                    binding, reserved,
                ))
            }
        }
    }

    fn complete_group_member(
        &self,
        binding: crate::physical_runtime::PhysicalDurabilityGroupMemberBinding,
        reserved: WalRangeReservedPhysicalMutation,
        expected_work: crate::physical_runtime::PhysicalWorkIdentity,
        expected_binding: crate::physical_runtime::PhysicalWalFrameCompletionBinding,
        settlement: PhysicalWalAppendSettlement,
        completed_bytes: u64,
    ) -> PhysicalWalGroupMemberAppendOutcome {
        let Some(settlement) =
            settlement.bind_completion(expected_work, reserved.artifact(), expected_binding)
        else {
            self.owner.seal_for_inspection();
            return PhysicalWalGroupMemberAppendOutcome::Indeterminate(WalBarrierMember::new(
                binding, reserved,
            ));
        };
        let persisted = reserved.persisted_attempt_binding();
        if self
            .owner
            .complete_member(
                reserved.resulting_frontier(),
                reserved.artifact().clone(),
                reserved.declaration(),
                completed_bytes,
                &self.idempotency,
                persisted,
            )
            .is_err()
        {
            return PhysicalWalGroupMemberAppendOutcome::Indeterminate(WalBarrierMember::new(
                binding, reserved,
            ));
        }
        PhysicalWalGroupMemberAppendOutcome::Appended(WalBarrierMember::new(
            binding,
            WalAppendedPhysicalMutation::new(reserved, settlement),
        ))
    }
}
