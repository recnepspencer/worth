//! Move one validated family source to its admitted lifecycle successor.
use super::super::super::registry::InstalledProducerProvider;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredRefreshClaim;
use std::any::TypeId;

/// The query proof keeps its value, source, request affinity and selected
/// observation. Only its executor guard moves after the typed family selector
/// has admitted the actual successor. Mutation authorization remains separate.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn bind_required_successor<
    Schema,
    Family,
>(
    mut fresh: FreshOutputDisclosure<
        FamilySourceQuery<Schema, Family>,
        FamilySourceValue<Schema, Family>,
    >,
    claim: &SelectedRequiredRefreshClaim,
    predecessor: &InstalledProducerProvider<Schema>,
    successor: &WorthQueryAdmittedOutputDemand<Schema, Family>,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    FreshOutputDisclosure<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
    WorthQueryOutputDemandDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let work = 2 * std::mem::size_of::<InstalledProducerEdition>()
        + 6 * std::mem::size_of::<TypeId>()
        + 12;
    admission
        .charge_external_work(
            u64::try_from(work)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?,
        )
        .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
    let installed = successor.installed_entry.as_ref();
    let same_source = predecessor.declaration.source_type == TypeId::of::<Family::Source>()
        && installed.declaration.source_type == TypeId::of::<Family::Source>();
    let same_family = claim.selected().key().family_identity() == Family::IDENTITY
        && predecessor.declaration.output_family_type == TypeId::of::<Family>()
        && installed.declaration.output_family_type == TypeId::of::<Family>();
    if fresh.0.edition != predecessor.edition || !same_source || !same_family {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "required successor does not own this family disclosure",
        ));
    }
    let before = claim.selected().key().applicability();
    let after = successor.selected.applicability;
    let producer_work = predecessor
        .declaration
        .identity
        .len()
        .checked_add(installed.declaration.identity.len())
        .and_then(|n| n.checked_add(before.profile_kind().len()))
        .and_then(|n| n.checked_add(after.profile_kind().len()))
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
    admission
        .charge_external_work(
            u64::try_from(producer_work)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?,
        )
        .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
    if predecessor.declaration.identity != installed.declaration.identity
        && (before.lifecycle() != WorthQueryProducerLifecyclePosture::Initial
            || after.lifecycle() != WorthQueryProducerLifecyclePosture::Preserve
            || before.profile_kind() != after.profile_kind())
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            "required disclosure has no declared lifecycle successor",
        ));
    }
    fresh.0.edition = installed.edition;
    Ok(fresh)
}
