use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPort, RuntimeWorldUnpublishedConditionalDefinition,
};

enum WorthQueryProductUnpublishedOwnerEffects {
    Application(ProductUnpublishedOwnerEffects),
    Recovered(worth_runtime_world::facade::WorldRecoveredUnpublishedEffects),
    ConditionalDefinition(RuntimeWorldUnpublishedConditionalDefinition),
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
    pub(in crate::domain_computation) fn retains_conditional_definition(&self) -> bool {
        matches!(
            self.effects,
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(_)
        )
    }

    pub(in crate::domain_computation) fn recovery_handle(
        &self,
    ) -> ProductUnpublishedRecoveryHandle {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.recovery_handle()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                value.effects().recovery_handle()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => {
                effects.recovery_handle()
            }
        }
    }
    pub(in crate::domain_computation) fn prepare_settled_relational_adoption(
        &self,
        cancellation: &worth_runtime_world::facade::RuntimeWorldCancellationToken,
        deadline: Option<worth_runtime_world::facade::RuntimeWorldInstant>,
    ) -> Result<
        worth_runtime_world::facade::PreparedCompositePublicationWithoutSignal,
        worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
    > {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => self
                .recovery
                .prepare_settled_relational_adoption(effects, cancellation, deadline),
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => self
                .recovery
                .prepare_settled_relational_adoption(value.effects(), cancellation, deadline),
            WorthQueryProductUnpublishedOwnerEffects::Recovered(_) => Err(
                worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                    RuntimeWorldRecoveryDenial::RecoveredCustodyRequired,
                ),
            ),
        }
    }

    pub(in crate::domain_computation) fn new_recovered(
        effects: worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            effects: WorthQueryProductUnpublishedOwnerEffects::Recovered(effects),
            recovery,
            disposition,
        }
    }
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
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => effects.cause(),
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.cause()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => {
                if effects.owner_effect_count() > 0 {
                    worth_runtime_world::facade::ProductUnpublishedCause::RecoveredWithoutProductMovement
                } else {
                    worth_runtime_world::facade::ProductUnpublishedCause::OpenAttemptRetained
                }
            }
        }
    }

    /// The typed refusal that stopped an original open before any owner effect.
    pub fn before_effect_refusal(&self) -> Option<worth_runtime_world::facade::NoEffectCause> {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.before_effect_refusal()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                value.effects().before_effect_refusal()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(_) => None,
        }
    }

    pub fn owner_effect_count(&self) -> usize {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.owner_effect_count()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.owner_effect_count()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => {
                effects.owner_effect_count()
            }
        }
    }

    /// Original correlation data, never permission to publish another attempt.
    pub fn attempt_identity(
        &self,
    ) -> &worth_runtime_world::facade::CompositePublicationAttemptIdentity {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.attempt_identity()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.attempt_identity()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => {
                effects.attempt_identity()
            }
        }
    }

    /// Immutable original Native meaning, when this record retains a full result.
    /// Borrowing it retains no snapshot or performed-publication capability.
    pub fn native_semantic_result(
        &self,
    ) -> Option<&worth_relational::facade::transactions::RelationalCommitSemanticResult> {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => effects
                .component_results()
                .relational_commit_result()
                .map(|result| result.semantic_result()),
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects
                    .component_results()
                    .relational_commit_result()
                    .map(|result| result.semantic_result())
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => effects
                .native_effect()
                .map(|effect| effect.semantic_result()),
        }
    }

    pub fn live_obligation_count(&self) -> usize {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.live_obligation_count()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.live_obligation_count()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => {
                effects.live_obligation_count()
            }
        }
    }

    pub fn expected_product(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.expected_head()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.expected_head()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => effects.expected_head(),
        }
    }

    pub fn next_actions(&self) -> &[worth_runtime_world::facade::ProductUnpublishedNextAction] {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.next_actions()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.next_actions()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(effects) => effects.next_actions(),
        }
    }

    pub fn relational_requires_settlement(&self) -> bool {
        match &self.effects {
            WorthQueryProductUnpublishedOwnerEffects::Application(effects) => {
                effects.progress().relational_requires_settlement()
            }
            WorthQueryProductUnpublishedOwnerEffects::ConditionalDefinition(value) => {
                let effects = value.effects();
                effects.progress().relational_requires_settlement()
            }
            WorthQueryProductUnpublishedOwnerEffects::Recovered(_) => false,
        }
    }

    pub fn into_recovery(self) -> super::WorthQueryProductUnpublishedRecovery {
        let handle = self.recovery_handle();
        drop(self.effects);
        super::WorthQueryProductUnpublishedRecovery::new(handle, self.recovery, self.disposition)
    }

    /// Refresh descriptive recovery facts through the same owning catalog.
    /// Inspection cannot publish, rerun effects, or promote this to a commit.
    pub fn inspect(&self) -> Result<Self, RuntimeWorldRecoveryDenial> {
        match self.recovery.inspect_effects(&self.recovery_handle()) {
            Ok(effects) => Ok(Self::new(
                effects,
                self.recovery.clone(),
                self.disposition.clone(),
            )),
            Err(RuntimeWorldRecoveryDenial::RecoveredCustodyRequired) => {
                Err(RuntimeWorldRecoveryDenial::CallerCapabilityLive)
            }
            Err(cause) => Err(cause),
        }
    }
}

impl std::fmt::Debug for WorthQueryProductUnpublishedApplication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductUnpublishedApplication")
            .field("handle", &self.recovery_handle())
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
#[path = "unpublished_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
