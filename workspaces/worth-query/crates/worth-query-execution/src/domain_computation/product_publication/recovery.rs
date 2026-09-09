use worth_runtime_world::facade::{
    OwnerRetirementWork, ProductUnpublishedRecoveryHandle, RecoveryContinuationContract,
    RuntimeWorldRecoveryDenial, RuntimeWorldRecoveryPort,
};

use super::WorthQueryProductUnpublishedApplication;

/// Exact access to one World-owned partial record. Holding this handle keeps no
/// strong effects view live and cannot authorize a committed application.
pub struct WorthQueryProductUnpublishedRecovery {
    handle: ProductUnpublishedRecoveryHandle,
    recovery: RuntimeWorldRecoveryPort,
}

impl WorthQueryProductUnpublishedRecovery {
    pub(super) fn new(
        handle: ProductUnpublishedRecoveryHandle,
        recovery: RuntimeWorldRecoveryPort,
    ) -> Self {
        Self { handle, recovery }
    }

    pub fn record_handle(&self) -> &ProductUnpublishedRecoveryHandle {
        &self.handle
    }

    pub fn inspect(
        &self,
    ) -> Result<WorthQueryProductUnpublishedApplication, RuntimeWorldRecoveryDenial> {
        self.recovery.inspect_effects(&self.handle).map(|effects| {
            WorthQueryProductUnpublishedApplication::new(effects, self.recovery.clone())
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

    /// Returns every actual retirement obligation emitted by World cleanup.
    #[must_use = "cleanup work must be retained or completed by its owning runtime"]
    pub fn release_obligations(
        &self,
        minimum_age_ticks: u64,
    ) -> Result<Vec<OwnerRetirementWork>, RuntimeWorldRecoveryDenial> {
        self.recovery
            .release_effects(&self.handle, minimum_age_ticks)
    }
}

impl std::fmt::Debug for WorthQueryProductUnpublishedRecovery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductUnpublishedRecovery")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}
