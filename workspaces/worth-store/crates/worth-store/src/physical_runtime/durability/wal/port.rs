#[cfg(feature = "certification-test-authority")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, Weak};

use worth_signal::facade::{AsyncNodeAdmissionClass, AsyncNodeConditionBlockClass};
use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::ArtifactTreeFailure;

use super::PhysicalWalRuntimeOwner;
use crate::physical_runtime::durability::PhysicalDurabilityGroupingRuntimeAuthority;
use crate::physical_runtime::durability::PhysicalMutationIdempotencyRuntimeAuthority;
use crate::physical_runtime::work::PhysicalWorkAdmissionAuthority;
use crate::physical_runtime::{
    instance::{PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime},
    record_serving::RecordWorkAdmission,
    PhysicalDurabilityObservation, PhysicalExecutorCommand, PhysicalExecutorCommandDenial,
    PhysicalSchedulerDenial, PhysicalWalAppendScope, PhysicalWalAppendSettlement,
    PhysicalWorkExecution, PhysicalWorkPreEffectDenial, PhysicalWorkSettlementEvidence,
    WalAppendedPhysicalMutation, WalBarrierMember, WalRangeReservedPhysicalMutation,
};

mod group;
mod maintenance;
pub(in crate::physical_runtime) use maintenance::ScheduledMaintenanceDenial;

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
    publication: Arc<
        OnceLock<Arc<crate::physical_runtime::durability::retention::PhysicalPublicationAdmission>>,
    >,
    #[cfg(feature = "certification-test-authority")]
    fail_next_member_before_effect: Arc<AtomicBool>,
    #[cfg(feature = "certification-test-authority")]
    owe_before_maintenance_barrier: Arc<AtomicBool>,
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
            publication: Arc::new(OnceLock::new()),
            #[cfg(feature = "certification-test-authority")]
            fail_next_member_before_effect: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "certification-test-authority")]
            owe_before_maintenance_barrier: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_owe_before_maintenance_barrier(&self) {
        self.owe_before_maintenance_barrier
            .store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn fail_next_member_before_effect(&self) {
        self.fail_next_member_before_effect
            .store(true, Ordering::Release);
    }

    pub(in crate::physical_runtime) fn bind_publication_admission(
        &self,
        admission: Arc<
            crate::physical_runtime::durability::retention::PhysicalPublicationAdmission,
        >,
    ) {
        let _ = self.publication.set(Arc::clone(&admission));
        self.owner.bind_publication_retention(admission);
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

    pub(in crate::physical_runtime) fn reopened_publications(&self) -> u64 {
        self.owner.reopened_publications()
    }

    pub(in crate::physical_runtime) fn plan_maintenance_frame(
        &self,
        payload: &[u8],
    ) -> Result<
        (
            worth_store_physical_backend::ArtifactTreeFile,
            u64,
            u64,
            u64,
            Vec<u8>,
        ),
        (),
    > {
        self.owner.plan_maintenance_frame(payload)
    }

    pub(in crate::physical_runtime) fn abort_maintenance_frame(&self) {
        self.owner.abort_maintenance_frame();
    }

    pub(in crate::physical_runtime) fn finish_maintenance_frame(&self) -> Result<(), ()> {
        self.owner.finish_maintenance_frame()
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
        #[cfg(feature = "certification-test-authority")]
        if self
            .fail_next_member_before_effect
            .swap(false, Ordering::AcqRel)
        {
            return Err(PhysicalWalAppendFailureCause::PreEffect(
                PhysicalWorkPreEffectDenial::ConsumerCancelled,
            ));
        }
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
        maintenance::prepare_wal_frame_command(
            self,
            reserved.artifact().clone(),
            reserved.encoded_frame().to_vec(),
            scope,
        )
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
