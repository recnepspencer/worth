use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RecoveryContinuationContract, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPort,
};

use super::WorthQueryProductUnpublishedApplication;

/// Exact access to one World-owned partial record. Holding this handle keeps no
/// strong effects view live and cannot authorize a committed application.
#[derive(Clone)]
pub struct WorthQueryProductUnpublishedRecovery {
    handle: ProductUnpublishedRecoveryHandle,
    recovery: RuntimeWorldRecoveryPort,
    disposition:
        crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
}

impl WorthQueryProductUnpublishedRecovery {
    pub(super) fn new(
        handle: ProductUnpublishedRecoveryHandle,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            handle,
            recovery,
            disposition,
        }
    }

    pub fn record_handle(&self) -> &ProductUnpublishedRecoveryHandle {
        &self.handle
    }

    pub fn inspect(
        &self,
    ) -> Result<WorthQueryProductUnpublishedApplication, RuntimeWorldRecoveryDenial> {
        self.recovery.inspect_effects(&self.handle).map(|effects| {
            WorthQueryProductUnpublishedApplication::new(
                effects,
                self.recovery.clone(),
                self.disposition.clone(),
            )
        })
    }

    /// Repairs only the owner settlement retained by this exact record.
    /// The returned actions cannot finish a sibling or publish product truth.
    pub fn continue_owner_settlement(
        &self,
    ) -> Result<RecoveryContinuationContract, RuntimeWorldRecoveryDenial> {
        let effects = self.recovery.inspect_effects(&self.handle)?;
        self.recovery.continue_effects(effects)
    }

    pub(super) fn into_release_parts(
        self,
    ) -> (
        ProductUnpublishedRecoveryHandle,
        RuntimeWorldRecoveryPort,
        crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) {
        (self.handle, self.recovery, self.disposition)
    }
}

/// Why releasing an unpublished product's recovery record was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductUnpublishedRecoveryReleaseDenial {
    /// Runtime World refused to inspect or release the record.
    World(RuntimeWorldRecoveryDenial),
    /// No owner-cleanup capacity was available; retry later.
    CleanupCapacityExhausted,
}

impl From<RuntimeWorldRecoveryDenial> for WorthQueryProductUnpublishedRecoveryReleaseDenial {
    fn from(denial: RuntimeWorldRecoveryDenial) -> Self {
        Self::World(denial)
    }
}

/// A refused recovery release, carrying the recovery record back unchanged.
///
/// [`Self::denial`] says why; [`Self::into_recovery`] returns the record so the
/// caller can retry.
#[derive(Debug)]
pub struct WorthQueryProductUnpublishedRecoveryFailure {
    denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
    recovery: WorthQueryProductUnpublishedRecovery,
}

impl WorthQueryProductUnpublishedRecoveryFailure {
    pub const fn denial(&self) -> WorthQueryProductUnpublishedRecoveryReleaseDenial {
        self.denial
    }

    pub fn into_recovery(self) -> WorthQueryProductUnpublishedRecovery {
        self.recovery
    }
}

/// Why `release_product_publication_recovery` did not return a cleanup
/// receipt.
#[derive(Debug)]
pub enum WorthQueryProductUnpublishedRecoveryReleaseFailure {
    /// The record was not released; the recovery record is returned.
    Recovery(WorthQueryProductUnpublishedRecoveryFailure),
    /// The record was released but owner cleanup did not finish; the cleanup is
    /// returned for retry.
    OwnerCleanup(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupFailure,
    ),
}

impl WorthQueryProductUnpublishedRecoveryReleaseFailure {
    pub fn into_recovery(self) -> Option<WorthQueryProductUnpublishedRecovery> {
        match self {
            Self::Recovery(failure) => Some(failure.into_recovery()),
            Self::OwnerCleanup(_) => None,
        }
    }

    pub fn into_owner_cleanup(
        self,
    ) -> Option<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup>
    {
        match self {
            Self::Recovery(_) => None,
            Self::OwnerCleanup(failure) => Some(failure.into_cleanup()),
        }
    }
}

pub(super) fn recovery_failure(
    recovery: WorthQueryProductUnpublishedRecovery,
    denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
) -> WorthQueryProductUnpublishedRecoveryReleaseFailure {
    WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
        WorthQueryProductUnpublishedRecoveryFailure { denial, recovery },
    )
}

impl std::fmt::Debug for WorthQueryProductUnpublishedRecovery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductUnpublishedRecovery")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}
