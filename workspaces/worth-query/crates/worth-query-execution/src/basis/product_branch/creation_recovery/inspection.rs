//! Read-only description of original branch-creation custody.
use super::*;
use worth_runtime_world::facade::{ProductUnpublishedCause, ProductUnpublishedNextAction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchCreationRecoveryCause {
    OpenAttemptRetained,
    OriginalObservationStopped(
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ),
    OriginalObservationUnavailable,
    WorldOccurrencePending,
    WorldOccurrenceRefused,
    WorldOccurrenceReadmissionRequired,
    SiblingOwnerDenied,
    CallerAbandoned,
    SettlementPending,
    CancellationAfterEffect,
    DeadlineAfterEffect,
    StaleProductHead,
    OwnerLost,
    CorrespondenceRebindRequired,
    DestinationAdmissionDenied,
    RetentionAdmissionDenied,
    ProductPublicationLost,
    /// The Relational owner could not establish whether its commit became
    /// durable; only reopening it resolves the commit's fate.
    RelationalDurabilityUnresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchCreationRecoveryNextAction {
    ContinueOriginalPublication,
    SettleOwnerEffects,
    ReleaseObligations,
    Inspect,
    ReopenRequired,
    StartFreshCompositePublication,
    CloseOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProductBranchCreationRecoveryInspection {
    cause: WorthQueryProductBranchCreationRecoveryCause,
    owner_effect_count: usize,
    live_obligation_count: usize,
    next_actions: Vec<WorthQueryProductBranchCreationRecoveryNextAction>,
}

impl WorthQueryProductBranchCreationRecoveryInspection {
    pub const fn cause(&self) -> WorthQueryProductBranchCreationRecoveryCause {
        self.cause
    }
    pub const fn owner_effect_count(&self) -> usize {
        self.owner_effect_count
    }
    pub const fn live_obligation_count(&self) -> usize {
        self.live_obligation_count
    }
    pub fn next_actions(&self) -> &[WorthQueryProductBranchCreationRecoveryNextAction] {
        &self.next_actions
    }
}

pub(super) fn inspection(
    effects: &ProductUnpublishedOwnerEffects,
) -> WorthQueryProductBranchCreationRecoveryInspection {
    WorthQueryProductBranchCreationRecoveryInspection {
        cause: map_cause(effects.cause()),
        owner_effect_count: effects.owner_effect_count(),
        live_obligation_count: effects.live_obligation_count(),
        next_actions: effects
            .next_actions()
            .iter()
            .copied()
            .map(map_action)
            .collect(),
    }
}

fn map_cause(cause: ProductUnpublishedCause) -> WorthQueryProductBranchCreationRecoveryCause {
    use ProductUnpublishedCause as Cause;
    match cause {
        Cause::RecoveredWithoutProductMovement => {
            unreachable!("cold original custody cannot construct live branch-creation effects")
        }
        Cause::OriginalObservationStopped(stop) => {
            WorthQueryProductBranchCreationRecoveryCause::OriginalObservationStopped(stop)
        }
        Cause::OriginalObservationUnavailable => {
            WorthQueryProductBranchCreationRecoveryCause::OriginalObservationUnavailable
        }
        Cause::WorldOccurrencePending => {
            WorthQueryProductBranchCreationRecoveryCause::WorldOccurrencePending
        }
        Cause::WorldOccurrenceRefused => {
            WorthQueryProductBranchCreationRecoveryCause::WorldOccurrenceRefused
        }
        Cause::WorldOccurrenceReadmissionRequired => {
            WorthQueryProductBranchCreationRecoveryCause::WorldOccurrenceReadmissionRequired
        }
        Cause::OpenAttemptRetained => {
            WorthQueryProductBranchCreationRecoveryCause::OpenAttemptRetained
        }
        Cause::SiblingOwnerDenied => {
            WorthQueryProductBranchCreationRecoveryCause::SiblingOwnerDenied
        }
        Cause::CallerAbandoned => WorthQueryProductBranchCreationRecoveryCause::CallerAbandoned,
        Cause::SettlementPending => WorthQueryProductBranchCreationRecoveryCause::SettlementPending,
        Cause::CancellationAfterEffect => {
            WorthQueryProductBranchCreationRecoveryCause::CancellationAfterEffect
        }
        Cause::DeadlineAfterEffect => {
            WorthQueryProductBranchCreationRecoveryCause::DeadlineAfterEffect
        }
        Cause::StaleProductHead => WorthQueryProductBranchCreationRecoveryCause::StaleProductHead,
        Cause::OwnerLost => WorthQueryProductBranchCreationRecoveryCause::OwnerLost,
        Cause::CorrespondenceRebindRequired => {
            WorthQueryProductBranchCreationRecoveryCause::CorrespondenceRebindRequired
        }
        Cause::DestinationAdmissionDenied => {
            WorthQueryProductBranchCreationRecoveryCause::DestinationAdmissionDenied
        }
        Cause::RetentionAdmissionDenied => {
            WorthQueryProductBranchCreationRecoveryCause::RetentionAdmissionDenied
        }
        Cause::ProductPublicationLost => {
            WorthQueryProductBranchCreationRecoveryCause::ProductPublicationLost
        }
        Cause::RelationalDurabilityUnresolved => {
            WorthQueryProductBranchCreationRecoveryCause::RelationalDurabilityUnresolved
        }
    }
}

fn map_action(
    action: ProductUnpublishedNextAction,
) -> WorthQueryProductBranchCreationRecoveryNextAction {
    use ProductUnpublishedNextAction as Action;
    match action {
        Action::ContinueOriginalPublication => {
            WorthQueryProductBranchCreationRecoveryNextAction::ContinueOriginalPublication
        }
        Action::SettleOwnerEffects => {
            WorthQueryProductBranchCreationRecoveryNextAction::SettleOwnerEffects
        }
        Action::ReleaseObligations => {
            WorthQueryProductBranchCreationRecoveryNextAction::ReleaseObligations
        }
        Action::Inspect => WorthQueryProductBranchCreationRecoveryNextAction::Inspect,
        Action::ReopenRequired => WorthQueryProductBranchCreationRecoveryNextAction::ReopenRequired,
        Action::StartFreshCompositePublication => {
            WorthQueryProductBranchCreationRecoveryNextAction::StartFreshCompositePublication
        }
        Action::CloseOwner => WorthQueryProductBranchCreationRecoveryNextAction::CloseOwner,
    }
}
