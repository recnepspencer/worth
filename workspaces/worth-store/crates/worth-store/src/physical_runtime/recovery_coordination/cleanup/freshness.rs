use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::{
    CompletedScheduledRecoveryReopenRead, DeniedScheduledRecoveryReopenRead,
    RecoveryReopenReadOutcome,
};
use worth_store_physical_format::{
    DurableRootSelector, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use crate::physical_runtime::recovery_coordination::settlement::{
    scheduler_posture, settle, signal_completion_is_terminal,
};
#[cfg(feature = "certification-test-authority")]
use crate::physical_runtime::recovery_coordination::settlement::{
    settle_with_certification, PhysicalRecoverySettlementCertificationStage,
};
use crate::physical_runtime::work::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload,
};
use crate::physical_runtime::PhysicalWorkSchedulerPosture;

use super::super::PhysicalRecoveryCoordination;

pub struct CompletedPhysicalRecoveryCleanupFreshnessRead {
    selector: DurableRootSelector,
    physical: CompletedScheduledRecoveryReopenRead,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

pub struct PhysicalRecoveryCleanupFreshnessReadDenial {
    kind: PhysicalRecoveryCleanupFreshnessReadDenialKind,
    progress: PhysicalRecoveryCleanupFreshnessReadProgress,
    integrity: Option<crate::physical_runtime::RootProtocolAdmissionDenial>,
}

#[derive(Default)]
pub struct PhysicalRecoveryCleanupFreshnessReadProgress {
    physical: Option<CompletedScheduledRecoveryReopenRead>,
    denied: Option<DeniedScheduledRecoveryReopenRead>,
    work: Option<crate::physical_runtime::PhysicalWorkIdentity>,
    scheduler: Option<PhysicalWorkSchedulerPosture>,
    signal: Option<crate::physical_runtime::PhysicalSignalSettlementOutcome>,
}

pub enum PhysicalRecoveryCleanupFreshnessReadDenialKind {
    Admission(super::admission::PhysicalRecoveryCleanupAdmissionDenial),
    Execution(crate::physical_runtime::PhysicalWorkPreEffectDenial),
    Media,
    SchedulerSettlement(PhysicalWorkSchedulerPosture),
    SignalSettlement(crate::physical_runtime::PhysicalSignalSettlementOutcome),
    Yieldpoint(crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult),
    InvalidSelector,
}

pub enum PhysicalRecoveryCleanupFreshnessReadOutcome {
    Completed(CompletedPhysicalRecoveryCleanupFreshnessRead),
    Denied(PhysicalRecoveryCleanupFreshnessReadDenial),
}

struct FreshnessReadExecution {
    dispatched: crate::physical_runtime::DispatchedPhysicalWork,
    plan: worth_store_io_scheduler::QueueExecutionReadyPlan,
    work: crate::physical_runtime::PhysicalWorkIdentity,
}

struct CompletedFreshnessSettlement {
    physical: CompletedScheduledRecoveryReopenRead,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    posture: PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

pub(super) fn read(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    format: PhysicalRecordFormatDeclaration,
) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
    let execution = match admit_execution(coordination) {
        Ok(execution) => execution,
        Err(outcome) => return outcome,
    };
    match media.read_recovery_artifact_scheduled(
        RecordArtifactFile::CurrentRootSelector,
        worth_store_physical_format::ROOT_SELECTOR_BYTES as u64,
        execution
            .plan
            .backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryReopenReadOutcome::Completed(completed) => {
            complete_read(coordination, media, format, execution, completed)
        }
        RecoveryReopenReadOutcome::Denied(physical) => {
            deny_media_read(coordination, media, format, execution, physical)
        }
    }
}

fn admit_execution(
    coordination: &PhysicalRecoveryCoordination,
) -> Result<FreshnessReadExecution, PhysicalRecoveryCleanupFreshnessReadOutcome> {
    let work = match super::admission::read(
        coordination,
        worth_store_physical_format::ROOT_SELECTOR_BYTES as u64,
    ) {
        Ok(work) => work,
        Err(denial) => {
            return Err(denied(
                PhysicalRecoveryCleanupFreshnessReadDenialKind::Admission(denial),
                PhysicalRecoveryCleanupFreshnessReadProgress::default(),
            ))
        }
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(denial) => {
            return Err(denied(
                PhysicalRecoveryCleanupFreshnessReadDenialKind::Execution(denial),
                PhysicalRecoveryCleanupFreshnessReadProgress {
                    work: Some(work_identity),
                    ..PhysicalRecoveryCleanupFreshnessReadProgress::default()
                },
            ))
        }
    };
    Ok(FreshnessReadExecution {
        dispatched,
        plan,
        work: work_identity,
    })
}

fn complete_read(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    format: PhysicalRecordFormatDeclaration,
    execution: FreshnessReadExecution,
    completed: CompletedScheduledRecoveryReopenRead,
) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
    let settlement = settle_completed_read(coordination, execution, completed);
    let kind = if settlement.posture != PhysicalWorkSchedulerPosture::Executed {
        Some(
            PhysicalRecoveryCleanupFreshnessReadDenialKind::SchedulerSettlement(settlement.posture),
        )
    } else if !signal_completion_is_terminal(settlement.signal) {
        Some(PhysicalRecoveryCleanupFreshnessReadDenialKind::SignalSettlement(settlement.signal))
    } else {
        None
    };
    if let Some(kind) = kind {
        return denied(kind, settlement.into_progress());
    }
    let admitted = match super::super::source_admission::admit_scheduled_current_selector(
        &settlement.physical,
        media.store_identity(),
        format,
    ) {
        Ok(admitted) => admitted,
        Err(integrity) => {
            return denied_integrity(
                PhysicalRecoveryCleanupFreshnessReadDenialKind::InvalidSelector,
                settlement.into_progress(),
                integrity,
            )
        }
    };
    let selector = match admitted.project() {
        Ok(selector) => selector,
        Err(integrity) => {
            return denied_integrity(
                PhysicalRecoveryCleanupFreshnessReadDenialKind::InvalidSelector,
                settlement.into_progress(),
                integrity,
            )
        }
    };
    coordination
        .root_protocol_counters
        .observe_selector(crate::physical_runtime::PhysicalRootProtocolRoute::CleanupFreshness);
    let wait = coordination
        .pause_at(crate::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead);
    if wait.is_interrupted() {
        return denied(
            PhysicalRecoveryCleanupFreshnessReadDenialKind::Yieldpoint(wait),
            completed_progress(
                settlement.physical.clone(),
                settlement.work,
                settlement.posture,
                settlement.signal,
            ),
        );
    }
    PhysicalRecoveryCleanupFreshnessReadOutcome::Completed(
        CompletedPhysicalRecoveryCleanupFreshnessRead {
            selector,
            physical: settlement.physical,
            work: settlement.work,
            signal: settlement.signal,
        },
    )
}

fn settle_completed_read(
    coordination: &PhysicalRecoveryCoordination,
    execution: FreshnessReadExecution,
    completed: CompletedScheduledRecoveryReopenRead,
) -> CompletedFreshnessSettlement {
    let queue = completed.queue();
    #[cfg(feature = "certification-test-authority")]
    let queue = if coordination.take_certification_cleanup_scheduler_failure(
        super::PhysicalRecoveryCleanupCommandStage::FreshnessRead,
    ) {
        queue.with_foreign_plan_binding_for_certification()
    } else {
        queue
    };
    let scheduler = execute_ready_queue_plan(execution.plan, queue);
    let posture = scheduler_posture(&scheduler);
    let dispatch = PhysicalExecutorDispatch::new(
        execution.dispatched,
        PhysicalExecutorOutcome::ReadCompleted {
            physical: completed.physical(),
            bytes: completed.bytes().into(),
            scheduler,
        },
        PhysicalEffectRecoveryObligation::Cleared,
    );
    #[cfg(feature = "certification-test-authority")]
    let signal = settle_with_certification(
        coordination,
        dispatch,
        PhysicalRecoverySettlementCertificationStage::Cleanup(
            super::PhysicalRecoveryCleanupCommandStage::FreshnessRead,
        ),
    );
    #[cfg(not(feature = "certification-test-authority"))]
    let signal = settle(coordination, dispatch);
    CompletedFreshnessSettlement {
        physical: completed,
        work: execution.work,
        posture,
        signal,
    }
}

fn deny_media_read(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    format: PhysicalRecordFormatDeclaration,
    execution: FreshnessReadExecution,
    physical: DeniedScheduledRecoveryReopenRead,
) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
    let absent =
        physical.failure().kind() == worth_store_physical_backend::ArtifactTreeFailureKind::Absent;
    let scheduler = physical
        .queue()
        .map(|queue| scheduler_posture(&execute_ready_queue_plan(execution.plan, queue)));
    let signal = settle(
        coordination,
        PhysicalExecutorDispatch::new(
            execution.dispatched,
            PhysicalExecutorOutcome::DeniedBeforeEffect {
                failure: physical.failure(),
                retry: PhysicalRetryPayload::Read,
            },
            PhysicalEffectRecoveryObligation::Cleared,
        ),
    );
    let progress = (
        PhysicalRecoveryCleanupFreshnessReadDenialKind::Media,
        PhysicalRecoveryCleanupFreshnessReadProgress {
            denied: Some(physical),
            work: Some(execution.work),
            scheduler,
            signal: Some(signal),
            ..PhysicalRecoveryCleanupFreshnessReadProgress::default()
        },
    );
    if absent {
        denied_integrity(
            progress.0,
            progress.1,
            crate::physical_runtime::RootProtocolAdmissionDenial::fixed_selector_absent(
                media.store_identity(),
                format,
                RecordArtifactFile::CurrentRootSelector,
            ),
        )
    } else {
        denied(progress.0, progress.1)
    }
}

fn denied(
    kind: PhysicalRecoveryCleanupFreshnessReadDenialKind,
    progress: PhysicalRecoveryCleanupFreshnessReadProgress,
) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
    PhysicalRecoveryCleanupFreshnessReadOutcome::Denied(
        PhysicalRecoveryCleanupFreshnessReadDenial {
            kind,
            progress,
            integrity: None,
        },
    )
}

fn denied_integrity(
    kind: PhysicalRecoveryCleanupFreshnessReadDenialKind,
    progress: PhysicalRecoveryCleanupFreshnessReadProgress,
    integrity: crate::physical_runtime::RootProtocolAdmissionDenial,
) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
    PhysicalRecoveryCleanupFreshnessReadOutcome::Denied(
        PhysicalRecoveryCleanupFreshnessReadDenial {
            kind,
            progress,
            integrity: Some(integrity),
        },
    )
}

fn completed_progress(
    physical: CompletedScheduledRecoveryReopenRead,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
) -> PhysicalRecoveryCleanupFreshnessReadProgress {
    PhysicalRecoveryCleanupFreshnessReadProgress {
        physical: Some(physical),
        denied: None,
        work: Some(work),
        scheduler: Some(scheduler),
        signal: Some(signal),
    }
}

impl CompletedFreshnessSettlement {
    fn into_progress(self) -> PhysicalRecoveryCleanupFreshnessReadProgress {
        completed_progress(self.physical, self.work, self.posture, self.signal)
    }
}

impl CompletedPhysicalRecoveryCleanupFreshnessRead {
    pub const fn selector(&self) -> DurableRootSelector {
        self.selector
    }
    pub const fn physical(&self) -> &CompletedScheduledRecoveryReopenRead {
        &self.physical
    }
    pub const fn work(&self) -> crate::physical_runtime::PhysicalWorkIdentity {
        self.work
    }
    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalSettlementOutcome {
        self.signal
    }
}

impl PhysicalRecoveryCleanupFreshnessReadDenial {
    pub const fn kind(&self) -> &PhysicalRecoveryCleanupFreshnessReadDenialKind {
        &self.kind
    }
    pub const fn completed(&self) -> Option<&CompletedScheduledRecoveryReopenRead> {
        self.progress.physical.as_ref()
    }
    pub const fn denied(&self) -> Option<&DeniedScheduledRecoveryReopenRead> {
        self.progress.denied.as_ref()
    }
    pub const fn progress(&self) -> &PhysicalRecoveryCleanupFreshnessReadProgress {
        &self.progress
    }
    pub const fn integrity(&self) -> Option<crate::physical_runtime::RootProtocolAdmissionDenial> {
        self.integrity
    }
}

impl PhysicalRecoveryCleanupFreshnessReadProgress {
    pub const fn work(&self) -> Option<crate::physical_runtime::PhysicalWorkIdentity> {
        self.work
    }
    pub const fn scheduler(&self) -> Option<PhysicalWorkSchedulerPosture> {
        self.scheduler
    }
    pub const fn signal(&self) -> Option<crate::physical_runtime::PhysicalSignalSettlementOutcome> {
        self.signal
    }
}
