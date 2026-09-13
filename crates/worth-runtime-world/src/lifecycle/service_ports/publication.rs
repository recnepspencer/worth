use super::super::availability::RuntimeWorldAvailability;
use super::super::{
    RuntimeWorldInstant, RuntimeWorldOwnerExecutionService, RuntimeWorldOwnerRoot,
    RuntimeWorldOwnerUnavailable, RuntimeWorldPreparationService,
};
use crate::branch::ProductBranchObservation;
use crate::publication::{
    CompositePublicationIntent, NoEffectCause, NoEffectCompositePublication,
    PreparedCompositePublicationWithSignal, PreparedCompositePublicationWithoutSignal,
    RuntimeWorldCancellationToken, RuntimeWorldPublicationOutcome, WithSignal, WithoutSignal,
};
use std::sync::{Arc, Weak};
use worth_signal::facade::{SignalError, SignalTransaction};

/// Executes the concrete component-owner progression and exact product CAS.
pub struct RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    owner: Weak<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>>,
}
impl<D, I, E, Ctx, T> Clone for RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
        }
    }
}
impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn new(owner: Weak<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>>) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>>, RuntimeWorldOwnerUnavailable> {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn prepare_without_signal(
        &self,
        expected: ProductBranchObservation,
        intent: CompositePublicationIntent<WithoutSignal>,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<PreparedCompositePublicationWithoutSignal, NoEffectCompositePublication> {
        let owner = self.service().map_err(|_| {
            NoEffectCompositePublication::new(
                NoEffectCause::OwnerUnavailable,
                Some(expected.clone()),
            )
        })?;
        owner.prepare_publication(expected, intent, cancellation, deadline)
    }
    pub fn prepare_with_signal(
        &self,
        expected: ProductBranchObservation,
        intent: CompositePublicationIntent<WithSignal>,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<PreparedCompositePublicationWithSignal, NoEffectCompositePublication> {
        let owner = self.service().map_err(|_| {
            NoEffectCompositePublication::new(
                NoEffectCause::OwnerUnavailable,
                Some(expected.clone()),
            )
        })?;
        owner.prepare_publication(expected, intent, cancellation, deadline)
    }
    pub fn execute_without_signal(
        &self,
        prepared: PreparedCompositePublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> RuntimeWorldPublicationOutcome {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => return unavailable(prepared.expected_head().clone()),
        };
        if prepared.expected_head().owner_identity() != owner.owner_identity() {
            return foreign(prepared.expected_head().clone());
        }
        let outcome = owner.execute_without_signal(prepared, cancellation);
        owner.finish_publication(outcome, cancellation)
    }
    pub fn execute_with_signal<F>(
        &self,
        prepared: PreparedCompositePublicationWithSignal,
        context: &mut Ctx,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
    ) -> RuntimeWorldPublicationOutcome
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => return unavailable(prepared.expected_head().clone()),
        };
        if prepared.expected_head().owner_identity() != owner.owner_identity() {
            return foreign(prepared.expected_head().clone());
        }
        let outcome = owner.execute_with_signal(prepared, context, cancellation, apply);
        owner.finish_publication(outcome, cancellation)
    }

    pub(crate) fn execute_conditional_definition_with_signal<F, H>(
        &self,
        mut prepared: PreparedCompositePublicationWithSignal,
        publication: worth_signal::facade::branch::SignalConditionalDefinitionPublicationOperation,
        context: &mut Ctx,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
        admit_activation: H,
    ) -> (
        RuntimeWorldPublicationOutcome,
        Arc<crate::publication::ConditionalDefinitionAttemptCustody>,
    )
    where
        F: FnOnce(
            &mut SignalTransaction<'_, D, I, E, Ctx, T>,
            &crate::publication::ConditionalDefinitionAttemptCustody,
        ) -> Result<(), SignalError>,
        H: FnOnce(
            worth_signal::facade::branch::SignalConditionalDefinitionAdvanceBinding,
            &crate::publication::ConditionalDefinitionAttemptCustody,
        ) -> Result<(), SignalError>,
    {
        let custody = prepared.reserve_conditional_definition_custody();
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => return (unavailable(prepared.expected_head().clone()), custody),
        };
        if prepared.expected_head().owner_identity() != owner.owner_identity() {
            return (foreign(prepared.expected_head().clone()), custody);
        }
        let outcome = owner.execute_conditional_definition_with_signal(
            prepared,
            publication,
            context,
            cancellation,
            |transaction| apply(transaction, &custody),
            |successor| admit_activation(successor, &custody),
        );
        (owner.finish_publication(outcome, cancellation), custody)
    }
}

impl RuntimeWorldPublicationPort<(), (), (), (), ()> {
    /// Publish and activate one Bridge definition as a single product-world
    /// operation. Bridge registry visibility is granted only after the exact
    /// product publication returns `Performed`.
    pub fn publish_bridge_conditional_definition(
        &self,
        prepared: PreparedCompositePublicationWithSignal,
        mut bridge_prepared: worth_runtime_bridge::facade::BridgePreparedConditionalInstallationExtension,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> crate::publication::RuntimeWorldConditionalDefinitionPublicationOutcome {
        use crate::publication::{
            RuntimeWorldConditionalDefinitionPublicationOutcome as Outcome,
            RuntimeWorldPublicationOutcome,
        };

        let publication = bridge_prepared.take_runtime_world_publication_operation();
        let (outcome, custody) = self.execute_conditional_definition_with_signal(
            prepared,
            publication,
            &mut (),
            cancellation,
            |transaction, custody| {
                let completion = bridge
                    .apply_owned_conditional_installation_extension(transaction, bridge_prepared)
                    .map_err(|denial| SignalError::invalid_input(format!("{denial:?}")))?;
                custody.retain_applied(completion);
                Ok(())
            },
            |binding, custody| {
                let mut applied = custody.lease_applied();
                bridge
                    .complete_owned_conditional_installation_activation(
                        applied.applied_mut(),
                        binding,
                    )
                    .map_err(|denial| SignalError::invalid_input(format!("{denial:?}")))?;
                Ok(())
            },
        );
        match outcome {
            RuntimeWorldPublicationOutcome::Performed(publication) => {
                let lowering =
                    bridge.commit_owned_conditional_installation_extension(custody.take_applied());
                Outcome::Performed {
                    publication,
                    lowering,
                }
            }
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => Outcome::NoEffect(no_effect),
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                Outcome::ProductUnpublished(
                    crate::publication::RuntimeWorldUnpublishedConditionalDefinition::new(effects),
                )
            }
        }
    }
}
fn unavailable(expected: ProductBranchObservation) -> RuntimeWorldPublicationOutcome {
    RuntimeWorldPublicationOutcome::NoEffect(NoEffectCompositePublication::new(
        NoEffectCause::OwnerUnavailable,
        Some(expected),
    ))
}
fn foreign(expected: ProductBranchObservation) -> RuntimeWorldPublicationOutcome {
    RuntimeWorldPublicationOutcome::NoEffect(NoEffectCompositePublication::new(
        NoEffectCause::OwnerDeniedBeforeEffect,
        Some(expected),
    ))
}
