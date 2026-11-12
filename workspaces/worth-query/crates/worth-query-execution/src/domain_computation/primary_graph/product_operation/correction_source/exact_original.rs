//! Resolve recorded coordinates by walking genuine current-owner history pages.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchObservation, RuntimeWorldObservationPort,
};

pub(super) fn resolve(
    request: ExecutionRequest<'_, '_>,
    observation: &RuntimeWorldObservationPort,
    target: &ProductBranchObservation,
    coordinates: &WorthQueryCorrectionSourceCoordinates,
    maximum: Option<NonZeroUsize>,
) -> Result<CompositeCommitIdentity, WorthQueryCorrectionSelectionDenial> {
    use WorthQueryCorrectionSelectionDenial as Denial;
    let mut cursor = observation
        .open_exact_product_history(request, target)
        .map_err(Denial::ExactHistory)?;
    let mut followed = 0_usize;
    loop {
        if maximum.is_some_and(|maximum| followed > maximum.get()) {
            return Err(Denial::HistoryLimit);
        }
        let page = observation
            .prepare_exact_product_history_page(request, &cursor)
            .map_err(Denial::ExactHistory)?;
        if let Some(commit) = page.commit() {
            let identity = commit.identity();
            let owner = match identity.owner_identity().durable() {
                Some(owner) => WorldOwnerCoordinate::Home { owner },
                None => WorldOwnerCoordinate::MemoryOnly {
                    observation: identity.owner_identity().get(),
                },
            };
            if owner == coordinates.world_owner()
                && identity.ordinal() == coordinates.world_commit()
            {
                if commit
                    .basis()
                    .relational_basis()
                    .observation()
                    .commit_receipt()
                    != Some(coordinates.relational())
                {
                    return Err(Denial::SourcePublicationMismatch);
                }
                return Ok(identity.clone());
            }
        }
        if page.is_complete() {
            return Err(Denial::SourceUnavailable);
        }
        let (_, next) = cursor
            .accept_page(page)
            .map_err(|cause| Denial::ExactHistory(RuntimeWorldServiceDenial::Denied(cause)))?;
        cursor = next;
        followed = followed.checked_add(1).ok_or(Denial::HistoryLimit)?;
    }
}

/// Resolve a canonical child through the protected original and actual history.
pub(super) fn performed<S: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    fact: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCorrectionFact,
    maximum: Option<NonZeroUsize>,
) -> Result<
    crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource,
    WorthQueryCorrectionSelectionDenial,
> {
    use WorthQueryCorrectionSelectionDenial as D;
    let owner = &target.application().runtime.product_runtime.owner;
    let originals = owner
        .inspection_port()
        .observe_program_activation_history(&execution)
        .map_err(D::HistorySource)?;
    let mut matching = originals
        .original_observations()
        .filter(|original| original.relational_commit() == Some(fact.child.commit_id));
    let original = matching.next().ok_or(D::SourceUnavailable)?;
    if matching.next().is_some() {
        return Err(D::SourcePublicationMismatch);
    }
    let coordinate = crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
        worth_foundational::facade::WorldOwnerCoordinate::Home { owner: original.identity().owner() },
        original.identity().ordinal(), fact.child.clone());
    let identity = resolve(
        execution,
        &owner.observation_port(),
        target.product().observation(),
        &coordinate,
        maximum,
    )?;
    Ok(crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource::from_untrusted_parts(identity, fact.child.clone()))
}
