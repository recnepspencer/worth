//! Terminal custody of a prepared provider commit; abort is a separate transition.
use super::{
    WorthQueryClosedProviderSessionDisposition, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionRecoveryPosture,
};

#[derive(Debug)]
pub enum WorthQuerySessionCommitOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    Committed(WorthQueryClosedProviderSessionDisposition),
    CommitDenied(WorthQueryProviderSessionFailure),
    CommitDeferred(super::WorthQueryProviderSessionCommitDeferred),
    CommitControlStopped(super::WorthQueryProviderSessionCommitControlStopped),
    CommitRecoveryRequired(WorthQueryProviderSessionFailure),
}

impl WorthQuerySessionCommitOutcome {
    pub fn recovery_posture(&self) -> WorthQueryProviderSessionRecoveryPosture {
        match self {
            Self::Committed(_)
            | Self::ProductStale(_)
            | Self::NoEffect(_)
            | Self::CommitDenied(_)
            | Self::CommitDeferred(_)
            | Self::CommitControlStopped(_) => WorthQueryProviderSessionRecoveryPosture::Closed,
            Self::ProductUnpublished(_) | Self::CommitRecoveryRequired(_) => {
                WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
            }
        }
    }

    pub fn failure(&self) -> Option<&WorthQueryProviderSessionFailure> {
        match self {
            Self::CommitDenied(failure) | Self::CommitRecoveryRequired(failure) => Some(failure),
            Self::Committed(_)
            | Self::ProductStale(_)
            | Self::NoEffect(_)
            | Self::ProductUnpublished(_)
            | Self::CommitDeferred(_)
            | Self::CommitControlStopped(_) => None,
        }
    }

    pub fn control_stopped(&self) -> Option<&super::WorthQueryProviderSessionCommitControlStopped> {
        match self {
            Self::CommitControlStopped(stopped) => Some(stopped),
            _ => None,
        }
    }
}
