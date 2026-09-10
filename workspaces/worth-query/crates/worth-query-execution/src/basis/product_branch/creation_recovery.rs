//! Query-owned custody for a branch creation whose owner effects did not publish.

use worth_runtime_world::facade::{
    ProductUnpublishedCause, ProductUnpublishedNextAction, ProductUnpublishedOwnerEffects,
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial, RuntimeWorldRecoveryPort,
};

pub use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupWork;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchCreationRecoveryCause {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchCreationRecoveryNextAction {
    SettleOwnerEffects,
    ReleaseObligations,
    Inspect,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchRecoveryDenial {
    OwnerUnavailable,
    ForeignRecovery,
    Missing,
    Busy,
    TooYoung,
    ClockRegressed,
    CallerCapabilityLive,
    SettlementRequired,
    SettlementEvidenceUnavailable,
    CleanupCapacityExhausted,
    NotBranchCreation,
}

#[derive(Debug)]
pub struct WorthQueryProductBranchCreationRecoveryRelease {
    retired_component_count: usize,
}

impl WorthQueryProductBranchCreationRecoveryRelease {
    pub const fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }
    pub const fn is_complete(&self) -> bool {
        true
    }
}

#[must_use = "creation recovery custody must be inspected, settled, or explicitly released"]
pub struct WorthQueryProductBranchCreationRecovery {
    effects: Option<ProductUnpublishedOwnerEffects>,
    handle: ProductUnpublishedRecoveryHandle,
    recovery: RuntimeWorldRecoveryPort,
    runtime: crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
}

impl WorthQueryProductBranchCreationRecovery {
    pub(crate) fn new(
        effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        runtime: crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Self {
        let handle = effects.recovery_handle();
        Self {
            effects: Some(effects),
            handle,
            recovery,
            runtime,
        }
    }

    pub fn inspect(
        &self,
    ) -> Result<
        WorthQueryProductBranchCreationRecoveryInspection,
        WorthQueryProductBranchRecoveryDenial,
    > {
        if let Some(effects) = self.effects.as_ref() {
            return Ok(inspection(effects));
        }
        self.recovery
            .inspect_effects(&self.handle)
            .map(|effects| inspection(&effects))
            .map_err(map_recovery_denial)
    }

    pub fn release_obligations(
        mut self,
        minimum_age_ticks: u64,
    ) -> Result<
        WorthQueryProductBranchCreationRecoveryRelease,
        WorthQueryProductBranchCreationRecoveryReleaseFailure,
    > {
        let cleanup_reservation = match self.runtime.reserve_owner_cleanup_for_creation() {
            Ok(reservation) => reservation,
            Err(_) => {
                return Err(
                    WorthQueryProductBranchCreationRecoveryReleaseFailure::Recovery(
                        WorthQueryProductBranchCreationRecoveryFailure {
                            denial: WorthQueryProductBranchRecoveryDenial::CleanupCapacityExhausted,
                            recovery: self,
                        },
                    ),
                );
            }
        };
        drop(self.effects.take());
        match self
            .recovery
            .release_effects(&self.handle, minimum_age_ticks)
        {
            Ok(work) => {
                let cleanup_identity = cleanup_reservation.install_unpublished(work);
                WorthQueryProductBranchOwnerCleanup::new(self.runtime.clone(), cleanup_identity)
                    .retry()
                    .map(|receipt| WorthQueryProductBranchCreationRecoveryRelease {
                        retired_component_count: receipt.retired_component_count(),
                    })
                    .map_err(WorthQueryProductBranchCreationRecoveryReleaseFailure::OwnerCleanup)
            }
            Err(denial) => Err(
                WorthQueryProductBranchCreationRecoveryReleaseFailure::Recovery(
                    WorthQueryProductBranchCreationRecoveryFailure {
                        denial: map_recovery_denial(denial),
                        recovery: self,
                    },
                ),
            ),
        }
    }
}

impl std::fmt::Debug for WorthQueryProductBranchCreationRecovery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductBranchCreationRecovery")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct WorthQueryProductBranchCreationRecoveryFailure {
    denial: WorthQueryProductBranchRecoveryDenial,
    recovery: WorthQueryProductBranchCreationRecovery,
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationRecoveryReleaseFailure {
    Recovery(WorthQueryProductBranchCreationRecoveryFailure),
    OwnerCleanup(WorthQueryProductBranchOwnerCleanupFailure),
}

impl WorthQueryProductBranchCreationRecoveryReleaseFailure {
    pub fn into_recovery(self) -> Option<WorthQueryProductBranchCreationRecovery> {
        match self {
            Self::Recovery(failure) => Some(failure.into_recovery()),
            Self::OwnerCleanup(_) => None,
        }
    }

    pub fn into_owner_cleanup(self) -> Option<WorthQueryProductBranchOwnerCleanup> {
        match self {
            Self::Recovery(_) => None,
            Self::OwnerCleanup(failure) => Some(failure.into_cleanup()),
        }
    }
}

impl WorthQueryProductBranchCreationRecoveryFailure {
    pub const fn denial(&self) -> WorthQueryProductBranchRecoveryDenial {
        self.denial
    }
    pub fn into_recovery(self) -> WorthQueryProductBranchCreationRecovery {
        self.recovery
    }
}

fn inspection(
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
    }
}

fn map_action(
    action: ProductUnpublishedNextAction,
) -> WorthQueryProductBranchCreationRecoveryNextAction {
    use ProductUnpublishedNextAction as Action;
    match action {
        Action::SettleOwnerEffects => {
            WorthQueryProductBranchCreationRecoveryNextAction::SettleOwnerEffects
        }
        Action::ReleaseObligations => {
            WorthQueryProductBranchCreationRecoveryNextAction::ReleaseObligations
        }
        Action::Inspect => WorthQueryProductBranchCreationRecoveryNextAction::Inspect,
        Action::StartFreshCompositePublication => {
            WorthQueryProductBranchCreationRecoveryNextAction::StartFreshCompositePublication
        }
        Action::CloseOwner => WorthQueryProductBranchCreationRecoveryNextAction::CloseOwner,
    }
}

pub(crate) fn map_recovery_denial(
    denial: RuntimeWorldRecoveryDenial,
) -> WorthQueryProductBranchRecoveryDenial {
    use RuntimeWorldRecoveryDenial as Denial;
    match denial {
        Denial::OwnerUnavailable(_) => WorthQueryProductBranchRecoveryDenial::OwnerUnavailable,
        Denial::ForeignHandle => WorthQueryProductBranchRecoveryDenial::ForeignRecovery,
        Denial::MissingRecord => WorthQueryProductBranchRecoveryDenial::Missing,
        Denial::Busy => WorthQueryProductBranchRecoveryDenial::Busy,
        Denial::TooYoung => WorthQueryProductBranchRecoveryDenial::TooYoung,
        Denial::ClockRegressed => WorthQueryProductBranchRecoveryDenial::ClockRegressed,
        Denial::CallerCapabilityLive => WorthQueryProductBranchRecoveryDenial::CallerCapabilityLive,
        Denial::SettlementRequired => WorthQueryProductBranchRecoveryDenial::SettlementRequired,
        Denial::SettlementEvidenceUnavailable => {
            WorthQueryProductBranchRecoveryDenial::SettlementEvidenceUnavailable
        }
        Denial::OutputCapacityExhausted => {
            WorthQueryProductBranchRecoveryDenial::CleanupCapacityExhausted
        }
    }
}
