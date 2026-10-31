//! Admit the actual cold metadata-only leaf without payload Query authority.
use crate::domain_computation::execution_resource_admission::{
    reserve_graph_provider_capacity_admitted, WorthQueryCapacityReservationAdmissionStop,
    WorthQueryExecutionResourceSupportSnapshot, WorthQueryGraphProviderLookupStop,
};
use crate::graph_obligation::{
    admitted_plan::WorthQueryGraphWorkAdmissionMechanics,
    selected_set::SelectedInstalledGraphObligations, WorthQueryAdmittedGraphWorkPlan,
    WorthQueryGraphObligationSelectionCounters, WorthQueryGraphWorkAdmissionDenial as Denial,
    WorthQueryGraphWorkCapacityAdmissionStop as Stop, WorthQueryGraphWorkIntent,
    WorthQuerySelectedGraphObligations,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInspectionContract,
    WorthQueryInstalledGraphObligationEffectPosture, WorthQueryInstalledGraphObligationSubjectKind,
};

/// The sealed binding supplies all inspection meaning; the caller supplies only
/// actual installed support and original-request work/storage admission.
pub fn admit_application_inspection_graph_work<Refusal>(
    contract: &WorthQueryInstalledApplicationInspectionContract,
    support: &WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Refusal>,
) -> Result<WorthQueryAdmittedGraphWorkPlan, Stop<Refusal>> {
    admit(1, 0).map_err(Stop::Admission)?;
    let obligations = contract.retain_graph_obligations_for_admission();
    let count = obligations.rows().len();
    admit(
        u64::try_from(count).map_err(|_| Stop::AccountingOverflow)?,
        0,
    )
    .map_err(Stop::Admission)?;
    if obligations.subject_kind()
        != WorthQueryInstalledGraphObligationSubjectKind::ApplicationInspection
        || obligations.rows().iter().any(|row| {
            matches!(
                row.effect_posture(),
                WorthQueryInstalledGraphObligationEffectPosture::Mutating
                    | WorthQueryInstalledGraphObligationEffectPosture::Invariant
            )
        })
    {
        return Err(Stop::Denial(Denial::InspectionAuthorityMismatch));
    }
    let selected = WorthQuerySelectedGraphObligations::seal(
        SelectedInstalledGraphObligations::ApplicationInspection(obligations),
        WorthQueryGraphWorkIntent::application_inspection_read(),
        WorthQueryGraphObligationSelectionCounters::retained_selection(count),
    );
    super::validate_owner_requirements(&selected).map_err(Stop::Denial)?;
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
        WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { capacity },
    )
    .ok_or(Stop::Denial(Denial::IdentityExhausted))
}
