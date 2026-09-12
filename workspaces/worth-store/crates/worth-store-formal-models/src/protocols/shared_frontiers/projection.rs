use crate::{
    CompactionVisibilityAction, DurabilityRecoveryAction, ImportPublicationAction,
    LeaseReclaimAction, QuarantineReadmissionState, ReplicationAdmissionAction,
    SourcePrecedenceAction,
};

use super::SharedFrontierAction;

pub const fn compose_durability_action(
    action: DurabilityRecoveryAction,
) -> Option<SharedFrontierAction> {
    match action {
        DurabilityRecoveryAction::CheckpointDurable
        | DurabilityRecoveryAction::DirectorySyncCompleted => {
            Some(SharedFrontierAction::DurabilityAdmitted)
        }
        DurabilityRecoveryAction::CheckpointPublished => {
            Some(SharedFrontierAction::CheckpointPublicationRequested)
        }
        DurabilityRecoveryAction::Crash => Some(SharedFrontierAction::Crash),
        DurabilityRecoveryAction::Reopen => Some(SharedFrontierAction::Reopen),
        _ => None,
    }
}

pub const fn compose_source_precedence_action(
    action: SourcePrecedenceAction,
) -> Option<SharedFrontierAction> {
    match action {
        SourcePrecedenceAction::SourceSelected => {
            Some(SharedFrontierAction::RecoveryPrecedencePreserved)
        }
        _ => None,
    }
}

pub const fn compose_compaction_action(
    action: CompactionVisibilityAction,
) -> Option<SharedFrontierAction> {
    match action {
        CompactionVisibilityAction::ValidateReadPlanCutover => {
            Some(SharedFrontierAction::CompactionCutover)
        }
        CompactionVisibilityAction::DeferReclaim => Some(SharedFrontierAction::ReclaimDeferred),
        CompactionVisibilityAction::DrainReclaimAfterReadRelease => {
            Some(SharedFrontierAction::ReclaimReleased)
        }
        _ => None,
    }
}

pub const fn compose_lease_action(action: LeaseReclaimAction) -> Option<SharedFrontierAction> {
    match action {
        LeaseReclaimAction::LeaseAcquired { .. } => Some(SharedFrontierAction::LiveLeaseAcquired),
        LeaseReclaimAction::LeaseReleased { .. } => Some(SharedFrontierAction::LeaseReleased),
        LeaseReclaimAction::ReclaimDeniedByLiveLease => Some(SharedFrontierAction::ReclaimDeferred),
        LeaseReclaimAction::ReclaimAdmitted => Some(SharedFrontierAction::ReclaimReleased),
        LeaseReclaimAction::IdentityReuseAdmitted { .. } => {
            Some(SharedFrontierAction::GenerationReused)
        }
        _ => None,
    }
}

pub const fn compose_quarantine_state(
    state: QuarantineReadmissionState,
) -> Option<SharedFrontierAction> {
    match state {
        QuarantineReadmissionState::Sealed => Some(SharedFrontierAction::QuarantineSealed),
        QuarantineReadmissionState::RecoveryVerificationPending => {
            Some(SharedFrontierAction::QuarantineVerificationStarted)
        }
        QuarantineReadmissionState::Readmitted => Some(SharedFrontierAction::QuarantineReadmitted),
        _ => None,
    }
}

pub const fn compose_import_action(
    action: ImportPublicationAction,
) -> Option<SharedFrontierAction> {
    match action {
        ImportPublicationAction::CurrentScopeReadmitted => {
            Some(SharedFrontierAction::ImportAdmissionPending)
        }
        ImportPublicationAction::RecoveredArtifactAdmitted => {
            Some(SharedFrontierAction::ExternalDurabilityAdmitted)
        }
        ImportPublicationAction::PublicationDurable => {
            Some(SharedFrontierAction::ExternalPublicationRequested)
        }
        ImportPublicationAction::CrashBeforePublication => Some(SharedFrontierAction::Crash),
        _ => None,
    }
}

pub const fn compose_replication_action(
    action: ReplicationAdmissionAction,
) -> Option<SharedFrontierAction> {
    match action {
        ReplicationAdmissionAction::SourceAdmitted => {
            Some(SharedFrontierAction::ReplicationAdmissionPending)
        }
        ReplicationAdmissionAction::FreshProgressObserved
        | ReplicationAdmissionAction::ResumeProgressObserved => {
            Some(SharedFrontierAction::ExternalDurabilityAdmitted)
        }
        ReplicationAdmissionAction::FreshPublicationDurable
        | ReplicationAdmissionAction::ResumePublicationDurable => {
            Some(SharedFrontierAction::ExternalPublicationRequested)
        }
        ReplicationAdmissionAction::SourceEpochDivergenceDetected
        | ReplicationAdmissionAction::LineageDivergenceDetected
        | ReplicationAdmissionAction::ReplayOverlapDivergenceDetected => {
            Some(SharedFrontierAction::ReplicationDivergenceDetected)
        }
        _ => None,
    }
}
