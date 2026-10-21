#[cfg(test)]
use super::WorthQueryAdmittedGraphReadAccessPlan;
use super::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessCaseRegistry,
    WorthQueryGraphReadAccessDenial,
};
#[cfg(test)]
use crate::runtime::worth_query_graph_index_inventory;
use crate::runtime::{
    explain_graph_read_access_requirements_for_family_in_authority_with_lookup,
    WorthQueryGraphIndexInventory, WorthQueryGraphReadAccessAuthorityContext,
    WorthQueryGraphReadAccessShapeExplanationError, WorthQueryGraphReadOperationLookup,
    WorthQueryReadFamily,
};
use worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReviewDenialKind;

#[cfg(test)]
pub fn admit_graph_read_access_for_family(
    family: &WorthQueryReadFamily,
) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError> {
    admit_graph_read_access_for_family_with_inventory(family, worth_query_graph_index_inventory())
}

#[cfg(test)]
pub(crate) fn admit_graph_read_access_for_family_with_inventory(
    family: &WorthQueryReadFamily,
    inventory: WorthQueryGraphIndexInventory,
) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError> {
    let authority = WorthQueryGraphReadAccessAuthorityContext::runtime_current_compatibility();
    admit_graph_read_access_for_family_in_authority_with_inventory(family, &authority, inventory)
}

#[cfg(test)]
pub(crate) fn admit_graph_read_access_for_family_in_authority_with_inventory(
    family: &WorthQueryReadFamily,
    authority: &WorthQueryGraphReadAccessAuthorityContext,
    inventory: WorthQueryGraphIndexInventory,
) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError> {
    admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup(
        family,
        authority,
        inventory,
        &crate::runtime::WorthQueryGraphReadOperationRegistry::empty(),
    )
}

pub(crate) fn admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup(
    family: &WorthQueryReadFamily,
    authority: &WorthQueryGraphReadAccessAuthorityContext,
    inventory: WorthQueryGraphIndexInventory,
    lookup: &impl WorthQueryGraphReadOperationLookup,
) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError> {
    let requirements = explain_graph_read_access_requirements_for_family_in_authority_with_lookup(
        family, authority, lookup,
    )?;
    let review =
        worth_query_admission::integration::review_graph_read_access(requirements, inventory);
    let case_registry = WorthQueryGraphReadAccessCaseRegistry::exhaustive();
    if let Some(denial) = review.denial() {
        let denial = map_denial(denial.kind());
        return Ok(WorthQueryGraphReadAccessAdmission::denied_in_authority(
            review.requirements().clone(),
            case_registry,
            review.inventory().clone(),
            review.inventory_match().clone(),
            authority.receipt().clone(),
            denial,
        ));
    }
    Ok(WorthQueryGraphReadAccessAdmission::admitted_in_authority(
        review.requirements().clone(),
        case_registry,
        review.inventory().clone(),
        review.inventory_match().clone(),
        authority.receipt().clone(),
        review.posture().clone(),
    ))
}

fn map_denial(kind: WorthQueryGraphReadPlanReviewDenialKind) -> WorthQueryGraphReadAccessDenial {
    match kind {
        WorthQueryGraphReadPlanReviewDenialKind::RequiredAsyncMaterialization => {
            WorthQueryGraphReadAccessDenial::required_async_materialization()
        }
        WorthQueryGraphReadPlanReviewDenialKind::RequiredAccessCapabilityRegistration => {
            WorthQueryGraphReadAccessDenial::required_access_capability_registration()
        }
        WorthQueryGraphReadPlanReviewDenialKind::RequiredPersistentIndex => {
            WorthQueryGraphReadAccessDenial::required_persistent_index()
        }
        WorthQueryGraphReadPlanReviewDenialKind::UnsupportedGraphIndexSupport => {
            WorthQueryGraphReadAccessDenial::unsupported_graph_index_support()
        }
    }
}

#[cfg(test)]
pub fn plan_admitted_graph_read_access_for_family(
    family: &WorthQueryReadFamily,
) -> Result<
    Option<WorthQueryAdmittedGraphReadAccessPlan>,
    WorthQueryGraphReadAccessShapeExplanationError,
> {
    let admission = admit_graph_read_access_for_family(family)?;
    Ok(WorthQueryAdmittedGraphReadAccessPlan::from_admission(
        admission,
    ))
}
