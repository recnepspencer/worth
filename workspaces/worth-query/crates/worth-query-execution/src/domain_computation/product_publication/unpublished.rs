use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, RuntimeWorldRecoveryDenial, RuntimeWorldRecoveryPort,
    RuntimeWorldUnpublishedConditionalDefinition,
};

enum WorthQueryProductUnpublishedOwnerEffects {
    Application(ProductUnpublishedOwnerEffects),
    ConditionalDefinition(RuntimeWorldUnpublishedConditionalDefinition),
}

impl WorthQueryProductUnpublishedOwnerEffects {
    fn effects(&self) -> &ProductUnpublishedOwnerEffects {
        match self {
            Self::Application(effects) => effects,
            Self::ConditionalDefinition(unpublished) => unpublished.effects(),
        }
    }
}

/// Owner effects retained by World after an application attempt failed to
/// publish its product occurrence. This value cannot authorize committed
/// projection, dispatch, or another attempt at the original effects.
#[must_use = "retain product-unpublished custody until its recovery obligations are resolved"]
pub struct WorthQueryProductUnpublishedApplication {
    effects: WorthQueryProductUnpublishedOwnerEffects,
    recovery: RuntimeWorldRecoveryPort,
    disposition:
        crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
}

impl WorthQueryProductUnpublishedApplication {
    pub(in crate::domain_computation) fn new(
        effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            effects: WorthQueryProductUnpublishedOwnerEffects::Application(effects),
            recovery,
            disposition,
        }
    }

    pub(in crate::domain_computation) fn new_conditional_definition(
        effects: RuntimeWorldUnpublishedConditionalDefinition,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            effects: WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(effects),
            recovery,
            disposition,
        }
    }

    pub fn cause(&self) -> worth_runtime_world::facade::ProductUnpublishedCause {
        self.effects.effects().cause()
    }

    pub fn owner_effect_count(&self) -> usize {
        self.effects.effects().owner_effect_count()
    }

    pub fn live_obligation_count(&self) -> usize {
        self.effects.effects().live_obligation_count()
    }

    pub fn expected_product(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        self.effects.effects().expected_head()
    }

    pub fn next_actions(&self) -> &[worth_runtime_world::facade::ProductUnpublishedNextAction] {
        self.effects.effects().next_actions()
    }

    pub fn relational_requires_settlement(&self) -> bool {
        self.effects
            .effects()
            .progress()
            .relational_requires_settlement()
    }

    pub fn into_recovery(self) -> super::WorthQueryProductUnpublishedRecovery {
        let handle = self.effects.effects().recovery_handle();
        drop(self.effects);
        super::WorthQueryProductUnpublishedRecovery::new(handle, self.recovery, self.disposition)
    }

    /// Refresh descriptive recovery facts through the same owning catalog.
    /// Inspection cannot publish, rerun effects, or promote this to a commit.
    pub fn inspect(&self) -> Result<Self, RuntimeWorldRecoveryDenial> {
        let effects = self
            .recovery
            .inspect_effects(&self.effects.effects().recovery_handle())?;
        Ok(Self::new(
            effects,
            self.recovery.clone(),
            self.disposition.clone(),
        ))
    }
}

impl std::fmt::Debug for WorthQueryProductUnpublishedApplication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductUnpublishedApplication")
            .field("effects", self.effects.effects())
            .finish_non_exhaustive()
    }
}
