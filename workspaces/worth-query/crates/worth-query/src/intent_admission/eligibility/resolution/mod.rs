use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod inspection;
mod nonruntime;
mod read;
mod routing;
mod runtime;

use super::artifact::WorthQueryIntentAdmissionPreDecisionPosture;
use super::facts::{
    WorthQueryIntentAdmissionAuthorityLaneEligibility, WorthQueryIntentAdmissionBasisEligibility,
    WorthQueryIntentAdmissionCapabilityEligibility, WorthQueryIntentAdmissionInvariantEligibility,
    WorthQueryIntentAdmissionPolicyEligibility,
    WorthQueryIntentAdmissionProjectionSourceEligibility,
    WorthQueryIntentAdmissionRoutingSupportEligibility,
    WorthQueryIntentAdmissionSourceLaneEligibility, WorthQueryIntentAdmissionSupportEligibility,
};
use super::request::WorthQueryRawIntentAdmissionRequest;
use crate::intent_admission::WorthQueryIntentAdmissionCoveredEntrypoint;
use inspection::resolve_inspection_materialization_eligibility;
use nonruntime::{
    resolve_basis_observation_eligibility, resolve_deferred_neighbor_eligibility,
    resolve_projection_consumption_eligibility,
};
use read::resolve_read_execution_eligibility;
use routing::resolve_existing_truth_probe_routing_eligibility;
use runtime::{
    resolve_authoritative_runtime_floor_eligibility, resolve_batch_write_floor_eligibility,
    resolve_effect_runtime_floor_eligibility, resolve_scalar_write_floor_eligibility,
};

pub(super) fn resolve_eligibility_facts(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> Result<
    (
        WorthQueryIntentAdmissionSupportEligibility,
        WorthQueryIntentAdmissionCapabilityEligibility,
        WorthQueryIntentAdmissionPolicyEligibility,
        WorthQueryIntentAdmissionBasisEligibility,
        WorthQueryIntentAdmissionInvariantEligibility,
        WorthQueryIntentAdmissionProjectionSourceEligibility,
        WorthQueryIntentAdmissionRoutingSupportEligibility,
        WorthQueryIntentAdmissionSourceLaneEligibility,
        WorthQueryIntentAdmissionAuthorityLaneEligibility,
        WorthQueryIntentAdmissionPreDecisionPosture,
    ),
    IdentityDerivationDenial,
> {
    Ok(match request.entrypoint() {
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteIntent => {
            resolve_authoritative_runtime_floor_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteNextEffectWriteIntent => {
            resolve_effect_runtime_floor_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite => {
            resolve_scalar_write_floor_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite => {
            resolve_batch_write_floor_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamily => {
            resolve_read_execution_eligibility(request, false, false)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamilyInBasisContext => {
            resolve_read_execution_eligibility(request, true, false)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead => {
            resolve_read_execution_eligibility(request, false, true)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection => {
            resolve_inspection_materialization_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteExistingTruthProbeRouting => {
            resolve_existing_truth_probe_routing_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::BasisObservation => {
            resolve_basis_observation_eligibility(request)
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ProjectionConsumption => {
            resolve_projection_consumption_eligibility(request)?
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteInspectionNeighborDeferred => {
            resolve_deferred_neighbor_eligibility(
                "inspection-materialization-neighbor-deferred-until-covered",
            )
        }
    })
}
