//! The owning World service admits and seals ordinary prepared operations.
use super::{
    NoEffectCause, NoEffectCompositePublication, ProductBranchObservation,
    ResolvedExpectedProductHead, RuntimeWorldCancellationToken, RuntimeWorldInstant,
    RuntimeWorldOwnerRoot,
};
use crate::publication::lower_component_plans;

impl<D, I, E, Ctx, T> crate::lifecycle::ports::RuntimeWorldPreparationService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn prepare_publication<S>(
        &self,
        expected: ProductBranchObservation,
        intent: crate::publication::CompositePublicationIntent<S>,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<S::Prepared, NoEffectCompositePublication>
    where
        S: crate::publication::CompositePublicationStage,
    {
        let current = self.admit_publication_source(&expected)?;
        // A directory World must durably open its original attempt before
        // Native seals a candidate; the memory entry grants no such phase.
        if self.state.durable.is_some() {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::DurableOpenRequired,
                Some(expected),
            ));
        }
        let (
            component_intent,
            prepared_candidate,
            settled_adoption,
            settled_signal_adoption,
            successor_observation_requested,
            current_read,
        ) = intent.into_parts();
        if let Some(current) = &current_read {
            self.admit_current_read_requirement(current, &expected)?;
        }
        let resolved = match ResolvedExpectedProductHead::from_current(
            component_intent,
            expected.clone(),
            &current,
        ) {
            Ok(resolved) => resolved,
            Err(_) => {
                drop(prepared_candidate);
                drop(settled_adoption);
                return Err(NoEffectCompositePublication::new(
                    NoEffectCause::StaleExpectedProductHead,
                    Some(expected),
                ));
            }
        };
        let plan = lower_component_plans(
            resolved,
            prepared_candidate,
            settled_adoption,
            settled_signal_adoption,
            current_read,
        )?;
        let attempt = RuntimeWorldOwnerRoot::reserve(
            self,
            plan,
            cancellation,
            deadline,
            successor_observation_requested,
        )?;
        Ok(S::seal(attempt))
    }

    fn prepare_creation(
        &self,
        source: ProductBranchObservation,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<
        crate::publication::ReservedBranchCreationAttempt,
        crate::branch::RuntimeWorldBranchAdmissionDenial,
    > {
        RuntimeWorldOwnerRoot::prepare_creation(self, source, intent, cancellation, deadline)
    }
}
