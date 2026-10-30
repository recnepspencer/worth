//! Distinct installed provider purpose; neither Source nor Parent Query permission substitutes.
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
use worth_query_installation::facade::{
    WorthQueryInstalledGraphObligationSubjectKind, WorthQueryInstalledManagedIndexAccessContract,
    WorthQueryInstalledManagedIndexParent,
};

/// Installed association and support, with no live provider-capacity reservation.
pub struct WorthQueryPreparedManagedIndexReadGraphWork {
    contract: Arc<WorthQueryInstalledManagedIndexAccessContract>,
    support: WorthQueryExecutionResourceSupportSnapshot,
}

pub fn prepare_managed_index_read_graph_work<Refusal>(
    parent: &WorthQueryInstalledManagedIndexParent,
    support: &WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Refusal>,
) -> Result<WorthQueryPreparedManagedIndexReadGraphWork, Stop<Refusal>> {
    let contract = parent.access_contract();
    let rows = u64::try_from(contract.graph_obligations().rows().len())
        .ok()
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::AccountingOverflow)?;
    admit(rows, 0).map_err(Stop::Admission)?;
    let selected = select_core(
        SelectedInstalledGraphObligations::PreparedManagedIndex(
            contract.retain_graph_obligations_for_admission(),
        ),
        WorthQueryGraphWorkIntent::managed_index_read(),
    )
    .map_err(|cause| Stop::Denial(Denial::Selection(cause)))?;
    super::validate_owner_requirements(&selected).map_err(Stop::Denial)?;
    if provider(support, admit)?.is_none() {
        return Err(Stop::Denial(Denial::ProviderSupportUnavailable));
    }
    Ok(WorthQueryPreparedManagedIndexReadGraphWork {
        contract: Arc::clone(contract),
        support: support.clone(),
    })
}

pub fn admit_prepared_managed_index_read_graph_work<Refusal>(
    parent: &WorthQueryInstalledManagedIndexParent,
    prepared: &WorthQueryPreparedManagedIndexReadGraphWork,
    support: &WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Refusal>,
) -> Result<WorthQueryAdmittedGraphWorkPlan, Stop<Refusal>> {
    admit(3, 0).map_err(Stop::Admission)?;
    let contract = &prepared.contract;
    let obligations = contract.retain_graph_obligations_for_admission();
    if !parent.owns_access_contract(contract)
        || obligations.subject_kind()
            != WorthQueryInstalledGraphObligationSubjectKind::ApplicationManagedIndex
        || obligations.binding_identity() != parent.binding_identity()
    {
        return Err(Stop::Denial(Denial::ManagedIndexAuthorityMismatch));
    }
    if !prepared.support.has_same_installed_authority(support) {
        return Err(Stop::Denial(Denial::ProviderSupportUnavailable));
    }
    let selected = WorthQuerySelectedGraphObligations::seal(
        SelectedInstalledGraphObligations::PreparedManagedIndex(obligations),
        WorthQueryGraphWorkIntent::managed_index_read(),
        WorthQueryGraphObligationSelectionCounters::retained_selection(
            contract.graph_obligations().rows().len(),
        ),
    );
    let provider =
        provider(support, admit)?.ok_or(Stop::Denial(Denial::ProviderSupportUnavailable))?;
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
        WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { capacity },
    )
    .ok_or(Stop::Denial(Denial::IdentityExhausted))
}

fn provider<'a, Refusal>(
    support: &'a WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Refusal>,
) -> Result<Option<&'a crate::domain_computation::execution_resource_admission::WorthQueryExecutionResourceSupport>, Stop<Refusal>>{
    support
        .graph_provider_admitted("primary", admit)
        .map_err(|cause| match cause {
            WorthQueryGraphProviderLookupStop::Admission(cause) => Stop::Admission(cause),
            WorthQueryGraphProviderLookupStop::AccountingOverflow => Stop::AccountingOverflow,
        })
}
