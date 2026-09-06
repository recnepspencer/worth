use crate::branch::{
    CustodyComponent, LoweredBranchCreationPlan, ProductBranchCreationIntent,
    ProductBranchObservation, ReservedCustodySlot, RuntimeWorldBranchAdmissionDenial,
};
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::{
    NoEffectCause, ReservedAttemptCapacities, ReservedAttemptCapacityInputs,
    ReservedBranchCreationAttempt, ReservedBranchCreationInputs, RuntimeWorldCancellationToken,
};

use super::reservation_steps::{
    issue_publication_identities, reserve_publication_resources, IssuedPublicationIdentities,
    ReservedPublicationResources,
};
use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Lower and reserve one branch creation. Every bounded resource the
    /// creation can consume, including one custody slot per owner fork it will
    /// ask for, is charged here before the first owner effect.
    pub(super) fn prepare_creation(
        &self,
        source: ProductBranchObservation,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<ReservedBranchCreationAttempt, RuntimeWorldBranchAdmissionDenial> {
        let plan = self.admit_creation_source(&source, intent, cancellation, deadline)?;
        let attempt = self.reserve_creation_attempt(source, plan, deadline)?;
        // The last boundary before the caller may ask an owner to fork.
        // Dropping the attempt on a denial releases every capacity it holds.
        creation_pre_effect_denial(self, cancellation, deadline)?;
        Ok(attempt)
    }

    /// The pre-reservation admission for a creation: this owner, an open and
    /// bootstrapped world, an intent that carries both per-owner postures, and
    /// an admitted source that still matches the registry's current head for
    /// that branch on every observation axis.
    ///
    /// The head comparison is taken after lowering, between the plan's
    /// admitted source and the live reference cell for that branch. Its
    /// independence comes from the cell: the caller supplies the source, the
    /// registry supplies what is current, and a caller holding an observation
    /// the cell has since moved past is refused here, before any capacity is
    /// charged.
    fn admit_creation_source(
        &self,
        source: &ProductBranchObservation,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<LoweredBranchCreationPlan, RuntimeWorldBranchAdmissionDenial> {
        if source.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.is_open_and_bootstrapped_for_creation() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        creation_pre_effect_denial(self, cancellation, deadline)?;
        let plan = LoweredBranchCreationPlan::lower(source.clone(), intent)?;
        self.admit_source_head(plan.expected())?;
        Ok(plan)
    }

    /// Charge every bounded resource the lowered plan can consume. Nothing
    /// here contacts a component owner.
    fn reserve_creation_attempt(
        &self,
        source: ProductBranchObservation,
        plan: LoweredBranchCreationPlan,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<ReservedBranchCreationAttempt, RuntimeWorldBranchAdmissionDenial> {
        let observation_capacity = self
            .state
            .retention
            .reserve_observation()
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::CapacityExhausted)?;
        let operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let IssuedPublicationIdentities {
            attempt_identity,
            commit_identity,
            product_unpublished_identity,
        } = issue_publication_identities(self).map_err(map_reservation_cause)?;
        let ReservedPublicationResources {
            history,
            reserved_commit_capacity,
            reserved_recovery_slot,
            reserved_component_pin_pair,
            reserved_publication_capacity,
        } = reserve_publication_resources(self, &source, &commit_identity, None)
            .map_err(map_reservation_cause)?;
        let relational_custody = reserve_custody(
            self,
            CustodyComponent::Relational,
            plan.relational().requires_owner_effect(),
        )?;
        let signal_custody = reserve_custody(
            self,
            CustodyComponent::Signal,
            plan.signal().requires_owner_effect(),
        )?;
        Ok(ReservedBranchCreationAttempt::new(
            ReservedBranchCreationInputs {
                identity: attempt_identity,
                source,
                plan,
                capacities: ReservedAttemptCapacities::new(ReservedAttemptCapacityInputs {
                    admitted_at: self.state.clock.now(),
                    reserved_commit_identity: commit_identity,
                    product_unpublished_identity,
                    reserved_commit_capacity,
                    reserved_recovery_slot,
                    reserved_component_pin_pair,
                    reserved_publication_capacity,
                    history,
                    operation,
                }),
                observation_capacity,
                relational_custody,
                signal_custody,
                deadline,
            },
        ))
    }
}

fn reserve_custody<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    component: CustodyComponent,
    required: bool,
) -> Result<Option<ReservedCustodySlot>, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if !required {
        return Ok(None);
    }
    owner.state.custody.reserve(component).map(Some)
}

fn creation_pre_effect_denial<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    cancellation: &RuntimeWorldCancellationToken,
    deadline: Option<RuntimeWorldInstant>,
) -> Result<(), RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if cancellation.is_cancelled() {
        return Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect);
    }
    if owner.deadline_expired(deadline) {
        return Err(RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect);
    }
    Ok(())
}

fn map_reservation_cause(cause: NoEffectCause) -> RuntimeWorldBranchAdmissionDenial {
    match cause {
        NoEffectCause::CapacityExhausted => RuntimeWorldBranchAdmissionDenial::CapacityExhausted,
        NoEffectCause::ReferenceGenerationExhausted => {
            RuntimeWorldBranchAdmissionDenial::IdentityExhausted
        }
        _ => RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
    }
}
