//! Query-owned custody for a branch creation whose owner effects did not publish.

use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPort,
};

pub use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupWork;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupFailure,
};

mod inspection;
use inspection::inspection;
pub use inspection::{
    WorthQueryProductBranchCreationRecoveryCause,
    WorthQueryProductBranchCreationRecoveryInspection,
    WorthQueryProductBranchCreationRecoveryNextAction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchRecoveryDenial {
    ForeignPhase,
    OriginalSourceUnavailable,
    OriginalSourceMismatch,
    ReopenRequired,
    RecoveredCustodyRequired,
    Execution(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
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

    /// Release the original effects and cleanup within the admitted advancement.
    pub fn release_obligations_in_advancement(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        minimum_age_ticks: u64,
    ) -> Result<
        WorthQueryProductBranchCreationRecoveryRelease,
        WorthQueryProductBranchCreationRecoveryReleaseFailure,
    > {
        if let Err(denial) = phase.check_live_for(&self.runtime) {
            return Err(
                WorthQueryProductBranchCreationRecoveryReleaseFailure::Recovery(
                    WorthQueryProductBranchCreationRecoveryFailure {
                        denial: WorthQueryProductBranchRecoveryDenial::Execution(denial),
                        recovery: self,
                    },
                ),
            );
        }
        self.release_obligations_with(phase, minimum_age_ticks, |cleanup| {
            cleanup.retry_in_advancement(phase)
        })
    }

    fn release_obligations_with(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        minimum_age_ticks: u64,
        finish_cleanup: impl FnOnce(
            WorthQueryProductBranchOwnerCleanup,
        ) -> Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductBranchOwnerCleanupFailure,
        >,
    ) -> Result<
        WorthQueryProductBranchCreationRecoveryRelease,
        WorthQueryProductBranchCreationRecoveryReleaseFailure,
    > {
        let execution = match phase.execution_request_for(&self.runtime) {
            Ok(execution) => execution,
            Err(_) => {
                return Err(
                    WorthQueryProductBranchCreationRecoveryReleaseFailure::Recovery(
                        WorthQueryProductBranchCreationRecoveryFailure {
                            denial: WorthQueryProductBranchRecoveryDenial::ForeignPhase,
                            recovery: self,
                        },
                    ),
                )
            }
        };
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
            .release_effects(&execution, &self.handle, minimum_age_ticks, None)
        {
            Ok(released) => {
                let (_original, cleanup) = released.into_parts();
                let cleanup_identity = cleanup_reservation.install_unpublished(cleanup);
                finish_cleanup(WorthQueryProductBranchOwnerCleanup::new(
                    self.runtime.clone(),
                    cleanup_identity,
                ))
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

pub(crate) fn map_recovery_denial(
    denial: RuntimeWorldRecoveryDenial,
) -> WorthQueryProductBranchRecoveryDenial {
    use RuntimeWorldRecoveryDenial as Denial;
    match denial {
        Denial::ContinuationAlreadyCommitted
        | Denial::ContinuationAlreadyStarted
        | Denial::ReleaseAlreadyStarted => WorthQueryProductBranchRecoveryDenial::NotBranchCreation,
        Denial::RecoveredCustodyRequired => {
            WorthQueryProductBranchRecoveryDenial::RecoveredCustodyRequired
        }
        Denial::ReopenRequired => WorthQueryProductBranchRecoveryDenial::ReopenRequired,
        Denial::OwnerUnavailable(_) => WorthQueryProductBranchRecoveryDenial::OwnerUnavailable,
        Denial::ForeignHandle => WorthQueryProductBranchRecoveryDenial::ForeignRecovery,
        Denial::MissingRecord => WorthQueryProductBranchRecoveryDenial::Missing,
        Denial::Busy => WorthQueryProductBranchRecoveryDenial::Busy,
        Denial::TooYoung => WorthQueryProductBranchRecoveryDenial::TooYoung,
        Denial::ClockRegressed => WorthQueryProductBranchRecoveryDenial::ClockRegressed,
        Denial::CallerCapabilityLive => WorthQueryProductBranchRecoveryDenial::CallerCapabilityLive,
        Denial::OriginalSourceUnavailable => {
            WorthQueryProductBranchRecoveryDenial::OriginalSourceUnavailable
        }
        Denial::OriginalSourceMismatch => {
            WorthQueryProductBranchRecoveryDenial::OriginalSourceMismatch
        }
        Denial::SettlementRequired => WorthQueryProductBranchRecoveryDenial::SettlementRequired,
        Denial::SettlementEvidenceUnavailable => {
            WorthQueryProductBranchRecoveryDenial::SettlementEvidenceUnavailable
        }
        Denial::OutputCapacityExhausted => {
            WorthQueryProductBranchRecoveryDenial::CleanupCapacityExhausted
        }
    }
}
