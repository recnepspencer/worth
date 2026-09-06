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
