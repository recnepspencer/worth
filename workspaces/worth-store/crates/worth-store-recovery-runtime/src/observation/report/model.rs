use super::super::RecoveryReportCounters;
use crate::{PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryRefusalKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReportOutcome {
    Recovered,
    Refused,
    Blocked,
    PublicationIndeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReportRefusalCause {
    CancelledBeforeDiscovery,
    CancelledBeforeReconstruction,
    CancelledBeforeExecution,
    EntryBindingDrift,
    PersistedStoreAdmission,
    CoordinationUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReportBlockCause {
    DiscoveryLimit,
    MediaObservation,
    RootProtocol,
    Checkpoint,
    WalInventory,
    SourceSelection,
    BindingFreshness,
    PageAdmission,
    OperationReconciliation,
    RedoPlanning,
    Staging,
    Publication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReportDenialCause {
    Refused(RecoveryReportRefusalCause),
    Blocked(RecoveryReportBlockCause),
    PublicationSettlementIndeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryReportEnvelope {
    pub(super) outcome: RecoveryReportOutcome,
    pub(super) store: Option<[u8; 16]>,
    pub(super) root_generation: Option<u64>,
    pub(super) counters: RecoveryReportCounters,
    pub(super) denial_cause: Option<RecoveryReportDenialCause>,
}

impl RecoveryReportEnvelope {
    pub fn from_outcome(outcome: &PhysicalRecoveryOutcome) -> Self {
        match outcome {
            PhysicalRecoveryOutcome::Recovered(handoff) => {
                let cleanup = handoff.cleanup_posture().evidence().counters();
                Self {
                    outcome: RecoveryReportOutcome::Recovered,
                    store: Some(handoff.core().store_identity().bytes()),
                    root_generation: Some(handoff.core().root().generation()),
                    counters: RecoveryReportCounters::new(
                        handoff.core().recovery_effect_count(),
                        cleanup.actions_completed,
                        cleanup.actions_deferred,
                        handoff.planning_counters().peak_recovery_bytes(),
                    ),
                    denial_cause: None,
                }
            }
            PhysicalRecoveryOutcome::Refused(refusal) => Self {
                outcome: RecoveryReportOutcome::Refused,
                store: None,
                root_generation: None,
                counters: RecoveryReportCounters::new(refusal.recovery_effects(), 0, 0, 0),
                denial_cause: Some(RecoveryReportDenialCause::Refused(refusal_cause(
                    refusal.kind,
                ))),
            },
            PhysicalRecoveryOutcome::Blocked(block) => Self {
                outcome: RecoveryReportOutcome::Blocked,
                store: Some(block.store_identity().bytes()),
                root_generation: block.evidence().source_generation,
                counters: RecoveryReportCounters::new(
                    block.recovery_effects(),
                    0,
                    0,
                    block.evidence().planning_counters.map_or(
                        0,
                        worth_store_recovery_physics::RecoveryPlanningCounters::peak_recovery_bytes,
                    ),
                ),
                denial_cause: Some(RecoveryReportDenialCause::Blocked(block_cause(block.kind))),
            },
            PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => Self {
                outcome: RecoveryReportOutcome::PublicationIndeterminate,
                store: Some(indeterminate.store_identity().bytes()),
                root_generation: None,
                counters: RecoveryReportCounters::new(indeterminate.recovery_effects(), 0, 0, 0),
                denial_cause: Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate),
            },
        }
    }

    pub const fn outcome(&self) -> RecoveryReportOutcome {
        self.outcome
    }

    pub const fn store_identity(&self) -> Option<[u8; 16]> {
        self.store
    }

    pub const fn root_generation(&self) -> Option<u64> {
        self.root_generation
    }

    pub const fn denial_cause(&self) -> Option<RecoveryReportDenialCause> {
        self.denial_cause
    }

    pub const fn counters(&self) -> RecoveryReportCounters {
        self.counters
    }
}

fn refusal_cause(kind: PhysicalRecoveryRefusalKind) -> RecoveryReportRefusalCause {
    match kind {
        PhysicalRecoveryRefusalKind::CancelledBeforeDiscovery => {
            RecoveryReportRefusalCause::CancelledBeforeDiscovery
        }
        PhysicalRecoveryRefusalKind::CancelledBeforeReconstruction => {
            RecoveryReportRefusalCause::CancelledBeforeReconstruction
        }
        PhysicalRecoveryRefusalKind::CancelledBeforeExecution => {
            RecoveryReportRefusalCause::CancelledBeforeExecution
        }
        PhysicalRecoveryRefusalKind::EntryBindingDrift(_) => {
            RecoveryReportRefusalCause::EntryBindingDrift
        }
        PhysicalRecoveryRefusalKind::PersistedStoreAdmission(_) => {
            RecoveryReportRefusalCause::PersistedStoreAdmission
        }
        PhysicalRecoveryRefusalKind::CoordinationUnavailable => {
            RecoveryReportRefusalCause::CoordinationUnavailable
        }
    }
}

fn block_cause(kind: PhysicalRecoveryBlockKind) -> RecoveryReportBlockCause {
    match kind {
        PhysicalRecoveryBlockKind::DiscoveryLimit => RecoveryReportBlockCause::DiscoveryLimit,
        PhysicalRecoveryBlockKind::MediaObservation => RecoveryReportBlockCause::MediaObservation,
        PhysicalRecoveryBlockKind::RootProtocol => RecoveryReportBlockCause::RootProtocol,
        PhysicalRecoveryBlockKind::Checkpoint => RecoveryReportBlockCause::Checkpoint,
        PhysicalRecoveryBlockKind::WalInventory => RecoveryReportBlockCause::WalInventory,
        PhysicalRecoveryBlockKind::SourceSelection => RecoveryReportBlockCause::SourceSelection,
        PhysicalRecoveryBlockKind::BindingFreshness => RecoveryReportBlockCause::BindingFreshness,
        PhysicalRecoveryBlockKind::PageAdmission => RecoveryReportBlockCause::PageAdmission,
        PhysicalRecoveryBlockKind::OperationReconciliation => {
            RecoveryReportBlockCause::OperationReconciliation
        }
        PhysicalRecoveryBlockKind::RedoPlanning => RecoveryReportBlockCause::RedoPlanning,
        PhysicalRecoveryBlockKind::Staging => RecoveryReportBlockCause::Staging,
        PhysicalRecoveryBlockKind::Publication => RecoveryReportBlockCause::Publication,
    }
}
