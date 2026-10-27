//! The installed Resource/action/facet contract, bound to actual provider support.
use crate::domain_computation::execution_resource_admission::{
    reserve_graph_provider_capacity_admitted, WorthQueryCapacityReservationAdmissionStop,
    WorthQueryExecutionResourceSupportSnapshot, WorthQueryGraphProviderLookupStop,
};
use crate::graph_obligation::{
    admitted_plan::WorthQueryGraphWorkAdmissionMechanics,
    selected_set::SelectedInstalledGraphObligations, selection::select_core,
    WorthQueryAdmittedGraphWorkPlan, WorthQueryGraphObligationSelectionCounters,
    WorthQueryGraphWorkAdmissionDenial as Denial, WorthQueryGraphWorkCapacityAdmissionStop as Stop,
    WorthQueryGraphWorkIntent, WorthQuerySelectedGraphObligations,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_resource::ApplicationResourceBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationResource,
    WorthQueryInstalledGraphObligationSubjectKind, WorthQueryInstalledResourceAccessContract,
    WorthQueryResourceAccessPurpose,
};

/// Cold compiled correspondence and actual installed support; no live capacity.
pub struct WorthQueryPreparedResourceReadGraphWork {
    contract: Arc<WorthQueryInstalledResourceAccessContract>,
    support: WorthQueryExecutionResourceSupportSnapshot,
}

pub fn prepare_resource_read_graph_work<Schema, Binding>(
    resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
    purpose: WorthQueryResourceAccessPurpose,
    support: &WorthQueryExecutionResourceSupportSnapshot,
) -> Result<WorthQueryPreparedResourceReadGraphWork, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let contract = resource.retain_access_contract(purpose);
    let selected = select_core(
        SelectedInstalledGraphObligations::PreparedResource(
            contract.retain_graph_obligations_for_admission(),
        ),
        WorthQueryGraphWorkIntent::resource_read(),
    )
    .map_err(Denial::Selection)?;
    super::validate_owner_requirements(&selected)?;
    if support.graph_provider("primary").is_none() {
        return Err(Denial::ProviderSupportUnavailable);
    }
    Ok(WorthQueryPreparedResourceReadGraphWork {
        contract,
        support: support.clone(),
    })
}

pub fn admit_prepared_resource_read_graph_work<Schema, Binding, Refusal>(
    resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
    purpose: WorthQueryResourceAccessPurpose,
    prepared: &WorthQueryPreparedResourceReadGraphWork,
    support: &WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Refusal>,
) -> Result<WorthQueryAdmittedGraphWorkPlan, Stop<Refusal>>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    admit(1, 0).map_err(Stop::Admission)?;
    let contract = &prepared.contract;
    let obligations = contract.retain_graph_obligations_for_admission();
    // This exact immutable leaf carries the full schema/resource/contracts
    // correspondence established by the private typed catalog compiler.
    if purpose != contract.purpose()
        || !resource.owns_access_contract(contract)
        || obligations.subject_kind()
            != WorthQueryInstalledGraphObligationSubjectKind::ApplicationResource
    {
        return Err(Stop::Denial(Denial::ResourceAuthorityMismatch));
    }
    if !prepared.support.has_same_installed_authority(support) {
        return Err(Stop::Denial(Denial::ProviderSupportUnavailable));
    }
    let selected = WorthQuerySelectedGraphObligations::seal(
        SelectedInstalledGraphObligations::PreparedResource(obligations),
        WorthQueryGraphWorkIntent::resource_read(),
        WorthQueryGraphObligationSelectionCounters::retained_selection(
            contract.graph_obligations().rows().len(),
        ),
    );
    let provider = support
        .graph_provider_admitted("primary", admit)
        .map_err(|cause| match cause {
            WorthQueryGraphProviderLookupStop::Admission(cause) => Stop::Admission(cause),
            WorthQueryGraphProviderLookupStop::AccountingOverflow => Stop::AccountingOverflow,
        })?
        .ok_or(Stop::Denial(Denial::ProviderSupportUnavailable))?;
    let capacity = reserve_graph_provider_capacity_admitted(provider, admit)
        .map_err(|cause| match cause {
            WorthQueryCapacityReservationAdmissionStop::Admission(cause) => Stop::Admission(cause),
            WorthQueryCapacityReservationAdmissionStop::AccountingOverflow => {
                Stop::AccountingOverflow
            }
        })?
        .ok_or(Stop::Denial(Denial::CapacityUnavailable))?;
    admit(2, 0).map_err(Stop::Admission)?;
    WorthQueryAdmittedGraphWorkPlan::seal(
        selected,
        WorthQueryGraphWorkAdmissionMechanics::ResourceRead { capacity },
    )
    .ok_or(Stop::Denial(Denial::IdentityExhausted))
}
