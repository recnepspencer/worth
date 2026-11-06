//! Original recovery already owns its catalog slot; reserve only its missing commit.
use super::*;

pub(in crate::lifecycle::owner::operation) struct ReservedReadmittedCommitResources {
    pub(in crate::lifecycle::owner::operation) history: CompositeHistoryCatalog,
    pub(in crate::lifecycle::owner::operation) capacity:
        crate::history::ReservedCompositeCommitCapacity,
    pub(in crate::lifecycle::owner::operation) pins: ReservedComponentPinPairCapacity,
    pub(in crate::lifecycle::owner::operation) publication: ReservedPublicationAttemptCapacity,
}

pub(in crate::lifecycle::owner::operation) fn reserve_readmitted_resources<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    expected: &ProductBranchObservation,
    commit: &CompositeCommitIdentity,
) -> Result<ReservedReadmittedCommitResources, NoEffectCause>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let history = owner.state.history.clone();
    let parent =
        CompositeCommitParent::Ordinary(OrdinaryParent::new(expected.selected_commit().clone()));
    let capacity = history
        .reserve_readmitted_capacity(commit.clone(), parent)
        .map_err(|_| NoEffectCause::CapacityExhausted)?;
    let pins = reserve_component_pin_pair(owner)?;
    let publication = reserve_publication_capacity(owner)?;
    Ok(ReservedReadmittedCommitResources {
        history,
        capacity,
        pins,
        publication,
    })
}
