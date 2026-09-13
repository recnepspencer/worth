use crate::history::{CompositeCommitParent, CompositeHistoryCatalog, OrdinaryParent};
use crate::identity::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity,
    ProductUnpublishedOwnerEffectsIdentity,
};
use crate::publication::NoEffectCause;
use crate::recovery::{RecoveryCatalogDenial, ReservedProductUnpublishedSlot};
use crate::retention::ReservedComponentPinPairCapacity;

use super::{
    LoweredOwnerComponentPlan, ProductBranchObservation, ReservedAttemptCapacities,
    ReservedAttemptCapacityInputs, ReservedCompositePublicationAttempt,
    ReservedPublicationAttemptCapacity, RuntimeWorldInstant, RuntimeWorldOperationReservation,
    RuntimeWorldOwnerRoot,
};

pub(super) struct IssuedPublicationIdentities {
    pub(super) attempt_identity: CompositePublicationAttemptIdentity,
    pub(super) commit_identity: CompositeCommitIdentity,
    pub(super) product_unpublished_identity: ProductUnpublishedOwnerEffectsIdentity,
}

pub(super) struct ReservedPublicationResources {
    pub(super) history: CompositeHistoryCatalog,
    pub(super) reserved_commit_capacity: crate::history::ReservedCompositeCommitCapacity,
    pub(super) reserved_recovery_slot: ReservedProductUnpublishedSlot,
    pub(super) reserved_component_pin_pair: ReservedComponentPinPairCapacity,
    pub(super) reserved_successor_observation: Option<ReservedSuccessorObservationResources>,
    pub(super) reserved_publication_capacity: ReservedPublicationAttemptCapacity,
}

pub(super) struct ReservedSuccessorObservationResources {
    pub(super) capacity: crate::retention::ReservedObservationCapacity,
    pub(super) component_pin_pair: ReservedComponentPinPairCapacity,
}

pub(super) fn issue_publication_identities<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
) -> Result<IssuedPublicationIdentities, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let mut identities = owner
        .state
        .identities
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let attempt_identity = identities
        .publication_attempt()
        .map_err(|_| NoEffectCause::PreEffectFailure)?;
    let commit_identity = identities
        .composite_commit()
        .map_err(|_| NoEffectCause::PreEffectFailure)?;
    let product_unpublished_identity = identities
        .product_unpublished()
        .map_err(|_| NoEffectCause::PreEffectFailure)?;
    Ok(IssuedPublicationIdentities {
        attempt_identity,
        commit_identity,
        product_unpublished_identity,
    })
}

pub(super) fn reserve_publication_resources<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    expected: &crate::branch::ProductBranchObservation,
    commit_identity: &CompositeCommitIdentity,
    publication_attempt: Option<&CompositePublicationAttemptIdentity>,
    successor_observation_requested: bool,
) -> Result<ReservedPublicationResources, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let history = owner.state.history.clone();
    let reserved_commit_capacity =
        reserve_history(&history, expected, commit_identity, publication_attempt)?;
    let reserved_recovery_slot = reserve_recovery(owner)?;
    let reserved_component_pin_pair = reserve_component_pin_pair(owner)?;
    let reserved_successor_observation = successor_observation_requested
        .then(|| reserve_successor_observation(owner))
        .transpose()?;
    let reserved_publication_capacity = reserve_publication_capacity(owner)?;
    Ok(ReservedPublicationResources {
        history,
        reserved_commit_capacity,
        reserved_recovery_slot,
        reserved_component_pin_pair,
        reserved_successor_observation,
        reserved_publication_capacity,
    })
}

fn reserve_successor_observation<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
) -> Result<ReservedSuccessorObservationResources, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let capacity = owner
        .state
        .retention
        .reserve_observation()
        .map_err(|_| NoEffectCause::CapacityExhausted)?;
    let component_pin_pair = reserve_component_pin_pair(owner)?;
    Ok(ReservedSuccessorObservationResources {
        capacity,
        component_pin_pair,
    })
}

fn reserve_history(
    history: &CompositeHistoryCatalog,
    expected: &crate::branch::ProductBranchObservation,
    commit_identity: &CompositeCommitIdentity,
    publication_attempt: Option<&CompositePublicationAttemptIdentity>,
) -> Result<crate::history::ReservedCompositeCommitCapacity, NoEffectCause> {
    if let Some(attempt) = publication_attempt {
        return history
            .reserve_publication_capacity(
                commit_identity.clone(),
                attempt.clone(),
                expected.snapshot().clone(),
            )
            .map_err(|_| NoEffectCause::CapacityExhausted);
    }
    let parent =
        CompositeCommitParent::Ordinary(OrdinaryParent::new(expected.selected_commit().clone()));
    history
        .reserve_commit_capacity(commit_identity.clone(), parent)
        .map_err(|_| NoEffectCause::CapacityExhausted)
}

fn reserve_recovery<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
) -> Result<ReservedProductUnpublishedSlot, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    match owner
        .state
        .recovery
        .reserve_product_unpublished(owner.owner_identity())
    {
        Ok(slot) => Ok(slot),
        Err(RecoveryCatalogDenial::CapacityExhausted { .. }) => {
            Err(NoEffectCause::CapacityExhausted)
        }
        Err(RecoveryCatalogDenial::ForeignOwner { .. }) => Err(NoEffectCause::OwnerUnavailable),
    }
}

fn reserve_component_pin_pair<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
) -> Result<ReservedComponentPinPairCapacity, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    owner
        .state
        .retention
        .reserve_product_publication_pair()
        .map_err(|_| NoEffectCause::CapacityExhausted)
}

fn reserve_publication_capacity<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
) -> Result<ReservedPublicationAttemptCapacity, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    owner
        .state
        .publication_capacity
        .reserve()
        .map_err(|_| NoEffectCause::CapacityExhausted)
}

/// Everything one reserved attempt owns once every capacity is held. Passing
/// it as one value keeps the reservation boundary checks readable as a single
/// sequence instead of a long constructor call.
pub(super) struct ReservedAttemptAssembly {
    pub(super) admitted_at: RuntimeWorldInstant,
    pub(super) identities: IssuedPublicationIdentities,
    pub(super) resources: ReservedPublicationResources,
    pub(super) plan: LoweredOwnerComponentPlan,
    pub(super) expected_head: ProductBranchObservation,
    pub(super) deadline: Option<RuntimeWorldInstant>,
    pub(super) operation: RuntimeWorldOperationReservation,
}

/// Bind the issued identities and the held capacities into the linear attempt.
/// No check happens here: every boundary the attempt depends on was already
/// taken by the caller, in order.
pub(super) fn assemble_reserved_attempt(
    assembly: ReservedAttemptAssembly,
) -> ReservedCompositePublicationAttempt {
    let ReservedAttemptAssembly {
        admitted_at,
        identities:
            IssuedPublicationIdentities {
                attempt_identity,
                commit_identity,
                product_unpublished_identity,
            },
        resources:
            ReservedPublicationResources {
                history,
                reserved_commit_capacity,
                reserved_recovery_slot,
                reserved_component_pin_pair,
                reserved_successor_observation,
                reserved_publication_capacity,
            },
        plan,
        expected_head,
        deadline,
        operation,
    } = assembly;
    let (reserved_successor_observation_capacity, reserved_successor_observation_pin_pair) =
        reserved_successor_observation.map_or((None, None), |reserved| {
            (Some(reserved.capacity), Some(reserved.component_pin_pair))
        });
    ReservedCompositePublicationAttempt::new(
        attempt_identity,
        expected_head.clone(),
        expected_head.basis().clone(),
        plan,
        ReservedAttemptCapacities::new(ReservedAttemptCapacityInputs {
            admitted_at,
            reserved_commit_identity: commit_identity,
            product_unpublished_identity,
            reserved_commit_capacity,
            reserved_recovery_slot,
            reserved_component_pin_pair,
            reserved_successor_observation_capacity,
            reserved_successor_observation_pin_pair,
            reserved_publication_capacity,
            history,
            operation,
        }),
        deadline,
    )
}
