use crate::basis_lifecycle::{evaluate_basis_observation_eligibility, DeniedBasisCapabilityKind};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::{
    evaluate_projection_consumption_eligibility, ProjectionConsumptionEligibility,
};

use super::super::artifact::WorthQueryIntentAdmissionPreDecisionPosture;
use super::super::facts::{
    WorthQueryIntentAdmissionAuthorityLaneEligibility, WorthQueryIntentAdmissionBasisEligibility,
    WorthQueryIntentAdmissionCapabilityEligibility, WorthQueryIntentAdmissionInvariantEligibility,
    WorthQueryIntentAdmissionPolicyEligibility,
    WorthQueryIntentAdmissionProjectionSourceEligibility,
    WorthQueryIntentAdmissionRoutingSupportEligibility,
    WorthQueryIntentAdmissionSourceLaneEligibility, WorthQueryIntentAdmissionSupportEligibility,
};
use super::super::request::WorthQueryRawIntentAdmissionRequest;

pub(super) fn resolve_basis_observation_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> EligibilityFacts {
    let normalized = request
        .basis_observation()
        .expect("basis observation request must preserve normalized basis intent")
        .clone();
    match evaluate_basis_observation_eligibility(normalized) {
        Ok(_) => admitted_basis_observation_facts(),
        Err(denial) => denied_basis_observation_facts(match denial.denial_kind() {
            DeniedBasisCapabilityKind::PolicyMasked => "basis-policy-masked",
            DeniedBasisCapabilityKind::PreviewDrifted => "basis-preview-drifted",
            DeniedBasisCapabilityKind::TenantMismatched => "basis-tenant-schema-mismatch",
            DeniedBasisCapabilityKind::LowerRuntimeBindingMissing => {
                "basis-lower-runtime-binding-required"
            }
            DeniedBasisCapabilityKind::LowerRuntimeCapabilityUnsupported => {
                "basis-unsupported-lane"
            }
            DeniedBasisCapabilityKind::Inaccessible => "basis-visibility-denied",
            DeniedBasisCapabilityKind::SchemaIncompatible => "basis-schema-incompatible",
            DeniedBasisCapabilityKind::OperationIneligible => "basis-operation-ineligible",
            DeniedBasisCapabilityKind::HistoricalReplayUnsupported => {
                "basis-historical-replay-unsupported"
            }
            DeniedBasisCapabilityKind::BridgeAuthorityMismatch => "basis-bridge-authority-mismatch",
            DeniedBasisCapabilityKind::RelationalAuthorityMismatch => {
                "basis-relational-authority-mismatch"
            }
            DeniedBasisCapabilityKind::SignalObservationMissing => {
                "basis-signal-observation-missing"
            }
            DeniedBasisCapabilityKind::RuntimeSnapshotStale => "basis-runtime-snapshot-stale",
            DeniedBasisCapabilityKind::DurableOverclaim => "basis-durable-overclaim",
            DeniedBasisCapabilityKind::StoreBackedDeferred => "basis-store-backed-deferred",
        }),
    }
}

pub(super) fn resolve_projection_consumption_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> Result<EligibilityFacts, IdentityDerivationDenial> {
    let declaration = request
        .projection_consumption_declaration()
        .expect("projection request must preserve declaration");
    Ok(match evaluate_projection_consumption_eligibility(declaration)? {
        ProjectionConsumptionEligibility::Admitted(_) => (
            WorthQueryIntentAdmissionSupportEligibility::Admitted,
            WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
            WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
            WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
            WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
            WorthQueryIntentAdmissionProjectionSourceEligibility::ProjectionConsumptionAdmitted,
            no_execution_handoff_projection(),
            WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
            WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
            WorthQueryIntentAdmissionPreDecisionPosture::Admitted,
        ),
        ProjectionConsumptionEligibility::AdmittedWithWarnings(_, _) => (
            WorthQueryIntentAdmissionSupportEligibility::Admitted,
            WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
            WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
            WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
            WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
            WorthQueryIntentAdmissionProjectionSourceEligibility::ProjectionConsumptionAdmittedWithWarnings(
                "projection-consumption-warning-bearing-admission",
            ),
            no_execution_handoff_projection(),
            WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
            WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
            WorthQueryIntentAdmissionPreDecisionPosture::Admitted,
        ),
        ProjectionConsumptionEligibility::Deferred(_) => deferred_projection_facts(),
        ProjectionConsumptionEligibility::Denied(_)
        | ProjectionConsumptionEligibility::SourceMismatch(_) => violated_projection_facts(),
    })
}

pub(super) fn resolve_deferred_neighbor_eligibility(detail: &'static str) -> EligibilityFacts {
    (
        WorthQueryIntentAdmissionSupportEligibility::Deferred(detail),
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
        WorthQueryIntentAdmissionPolicyEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionBasisEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionInvariantEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionProjectionSourceEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionRoutingSupportEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionSourceLaneEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionAuthorityLaneEligibility::DeferredNeighbor(detail),
        WorthQueryIntentAdmissionPreDecisionPosture::Deferred {
            stage: "support-deferred",
            message: detail,
        },
    )
}

type EligibilityFacts = (
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
);

fn admitted_basis_observation_facts() -> EligibilityFacts {
    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::ObservationLifecycleAdmitted,
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionRoutingSupportEligibility::NoExecutionHandoff(
            "basis-observation-admitted-plan-scopes-to-lower-runtime-evidence-without-query-execution-handoff",
        ),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionPreDecisionPosture::Admitted,
    )
}

fn denied_basis_observation_facts(detail: &'static str) -> EligibilityFacts {
    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
            stage: "basis-observation-eligibility",
            detail,
        },
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::ObservationLifecycleViolation(detail),
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionRoutingSupportEligibility::NoExecutionHandoff(
            "basis-observation-admitted-plan-scopes-to-lower-runtime-evidence-without-query-execution-handoff",
        ),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionPreDecisionPosture::Violation {
            stage: "basis-observation-eligibility",
            message: detail,
        },
    )
}

fn deferred_projection_facts() -> EligibilityFacts {
    (
        WorthQueryIntentAdmissionSupportEligibility::Deferred("projection-consumption-deferred"),
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
        WorthQueryIntentAdmissionPolicyEligibility::DeferredNeighbor(
            "projection-consumption-deferred",
        ),
        WorthQueryIntentAdmissionBasisEligibility::DeferredNeighbor(
            "projection-consumption-deferred",
        ),
        WorthQueryIntentAdmissionInvariantEligibility::DeferredNeighbor(
            "projection-consumption-deferred",
        ),
        WorthQueryIntentAdmissionProjectionSourceEligibility::DeferredNeighbor(
            "projection-consumption-deferred",
        ),
        no_execution_handoff_projection(),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionPreDecisionPosture::Deferred {
            stage: "support-deferred",
            message: "projection-consumption-deferred",
        },
    )
}

fn violated_projection_facts() -> EligibilityFacts {
    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
            stage: "projection-consumption-eligibility",
            detail: "projection-consumption-violation",
        },
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::ProjectionConsumptionViolation(
            "projection-consumption-violation",
        ),
        no_execution_handoff_projection(),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionPreDecisionPosture::Violation {
            stage: "projection-consumption-eligibility",
            message: "projection-consumption-violation",
        },
    )
}

fn no_execution_handoff_projection() -> WorthQueryIntentAdmissionRoutingSupportEligibility {
    WorthQueryIntentAdmissionRoutingSupportEligibility::NoExecutionHandoff(
        "projection-consumption-admitted-plan-binds-contract-without-query-execution-handoff",
    )
}
