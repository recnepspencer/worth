use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, RuntimeWorldRecoveryDenial, RuntimeWorldRecoveryPort,
};

/// Owner effects retained by World after an application attempt failed to
/// publish its product occurrence. This value cannot authorize committed
/// projection, dispatch, or another attempt at the original effects.
#[must_use = "retain product-unpublished custody until its recovery obligations are resolved"]
pub struct WorthQueryProductUnpublishedApplication {
    effects: ProductUnpublishedOwnerEffects,
    recovery: RuntimeWorldRecoveryPort,
}

impl WorthQueryProductUnpublishedApplication {
    pub(in crate::domain_computation) fn new(
        effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
    ) -> Self {
        Self { effects, recovery }
    }

    pub fn cause(&self) -> worth_runtime_world::facade::ProductUnpublishedCause {
        self.effects.cause()
    }

    pub fn owner_effect_count(&self) -> usize {
        self.effects.owner_effect_count()
    }

    pub fn live_obligation_count(&self) -> usize {
        self.effects.live_obligation_count()
    }

    pub fn expected_product(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        self.effects.expected_head()
    }

    pub fn next_actions(&self) -> &[worth_runtime_world::facade::ProductUnpublishedNextAction] {
        self.effects.next_actions()
    }

    pub fn relational_requires_settlement(&self) -> bool {
        self.effects.progress().relational_requires_settlement()
    }

    pub fn into_recovery(self) -> super::WorthQueryProductUnpublishedRecovery {
        let handle = self.effects.recovery_handle();
        drop(self.effects);
        super::WorthQueryProductUnpublishedRecovery::new(handle, self.recovery)
    }

    /// Refresh descriptive recovery facts through the same owning catalog.
    /// Inspection cannot publish, rerun effects, or promote this to a commit.
    pub fn inspect(&self) -> Result<Self, RuntimeWorldRecoveryDenial> {
        let effects = self
            .recovery
            .inspect_effects(&self.effects.recovery_handle())?;
        Ok(Self::new(effects, self.recovery.clone()))
    }
}

impl std::fmt::Debug for WorthQueryProductUnpublishedApplication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductUnpublishedApplication")
            .field("effects", &self.effects)
            .finish_non_exhaustive()
    }
}
