use crate::runtime::{
    admit_authoritative_intent_declaration, admit_effect_triggered_intent_declaration,
    WorthQueryAuthorityLane, WorthQueryIntentAdmissionDenial, WorthQueryIntentSourceLane,
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
use super::super::seeds::WorthQueryAuthoritativeMutationPreflight;
use crate::intent_admission::WorthQueryIntentAdmissionExecutionSeam;

pub(super) fn resolve_authoritative_runtime_floor_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> EligibilityFacts {
    resolve_implemented_runtime_floor_eligibility(
        request,
        WorthQueryIntentSourceLane::UserAuthored,
        WorthQueryAuthorityLane::AuthoritativeTruth,
        admit_authoritative_intent_declaration(
            request
                .runtime_declaration()
                .expect("covered runtime request must preserve declaration"),
        ),
    )
}

pub(super) fn resolve_effect_runtime_floor_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> EligibilityFacts {
    resolve_implemented_runtime_floor_eligibility(
        request,
        WorthQueryIntentSourceLane::EffectTriggered,
        WorthQueryAuthorityLane::AuthoritativeTruth,
        admit_effect_triggered_intent_declaration(
            request
                .runtime_declaration()
                .expect("effect floor request must preserve declaration"),
        ),
    )
}

pub(super) fn resolve_scalar_write_floor_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> EligibilityFacts {
    let seed = request
        .authoritative_mutation_seed()
        .expect("scalar write request must preserve mutation seed");
    let (capability_posture, pre_decision_posture) = match seed.preflight() {
        WorthQueryAuthoritativeMutationPreflight::Admitted { .. } => (
            WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
            WorthQueryIntentAdmissionPreDecisionPosture::Admitted,
        ),
        WorthQueryAuthoritativeMutationPreflight::BindingDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "existing-truth-binding-admission",
                detail: "scalar-write-existing-truth-binding-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "existing-truth-binding-admission",
                message: "scalar-write-existing-truth-binding-denied",
            },
        ),
        WorthQueryAuthoritativeMutationPreflight::AssertionDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "verified-existing-truth-assertion",
                detail: "scalar-write-verified-existing-truth-assertion-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "verified-existing-truth-assertion",
                message: "scalar-write-verified-existing-truth-assertion-denied",
            },
        ),
        WorthQueryAuthoritativeMutationPreflight::ContinuityDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "mutation-continuity-admission",
                detail: "scalar-write-continuity-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "mutation-continuity-admission",
                message: "scalar-write-continuity-denied",
            },
        ),
        WorthQueryAuthoritativeMutationPreflight::ContractDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "native-mutation-contract-admission",
                detail: "scalar-write-native-contract-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "native-mutation-contract-admission",
                message: "scalar-write-native-contract-denied",
            },
        ),
        WorthQueryAuthoritativeMutationPreflight::NamingDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "mutation-naming-admission",
                detail: "scalar-write-naming-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "mutation-naming-admission",
                message: "scalar-write-naming-denied",
            },
        ),
        WorthQueryAuthoritativeMutationPreflight::TargetReferenceDenied(_) => (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "mutation-target-reference-admission",
                detail: "scalar-write-target-reference-denied",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "mutation-target-reference-admission",
                message: "scalar-write-target-reference-denied",
            },
        ),
    };

    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        capability_posture,
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            WorthQueryIntentAdmissionExecutionSeam::BackendWriteAuthorityRoute,
        ),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        pre_decision_posture,
    )
}

pub(super) fn resolve_batch_write_floor_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
) -> EligibilityFacts {
    let commands = request
        .authoritative_mutation_batch_seed()
        .expect("batch write request must preserve mutation batch seed")
        .commands();
    let (capability_posture, pre_decision_posture) = if commands.is_empty() {
        (
            WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                stage: "mutation-batch-authoring",
                detail: "batch-write-requires-at-least-one-command",
            },
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage: "mutation-batch-authoring",
                message: "batch-write-requires-at-least-one-command",
            },
        )
    } else {
        (
            WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
            WorthQueryIntentAdmissionPreDecisionPosture::Admitted,
        )
    };

    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        capability_posture,
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            WorthQueryIntentAdmissionExecutionSeam::BackendWriteAuthorityRoute,
        ),
        WorthQueryIntentAdmissionSourceLaneEligibility::NotApplicableNonRuntimeFamily,
        WorthQueryIntentAdmissionAuthorityLaneEligibility::NotApplicableNonRuntimeFamily,
        pre_decision_posture,
    )
}

pub(super) type EligibilityFacts = (
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

fn resolve_implemented_runtime_floor_eligibility(
    request: &WorthQueryRawIntentAdmissionRequest,
    expected_source_lane: WorthQueryIntentSourceLane,
    expected_target_lane: WorthQueryAuthorityLane,
    admission: Result<(), WorthQueryIntentAdmissionDenial>,
) -> EligibilityFacts {
    let source_lane_posture = lane_source_posture(request, expected_source_lane);
    let authority_lane_posture = lane_authority_posture(request, expected_target_lane);
    let capability_posture = match admission {
        Ok(()) => WorthQueryIntentAdmissionCapabilityEligibility::Admitted,
        Err(denial) => capability_violation_for_denial(request, denial.stage()),
    };
    let pre_decision_posture = match capability_posture {
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted => {
            WorthQueryIntentAdmissionPreDecisionPosture::Admitted
        }
        WorthQueryIntentAdmissionCapabilityEligibility::Violation { stage, detail } => {
            WorthQueryIntentAdmissionPreDecisionPosture::Violation {
                stage,
                message: detail,
            }
        }
    };

    (
        WorthQueryIntentAdmissionSupportEligibility::Admitted,
        capability_posture,
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired,
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor,
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute,
        ),
        source_lane_posture,
        authority_lane_posture,
        pre_decision_posture,
    )
}

fn lane_source_posture(
    request: &WorthQueryRawIntentAdmissionRequest,
    expected: WorthQueryIntentSourceLane,
) -> WorthQueryIntentAdmissionSourceLaneEligibility {
    let actual = request
        .runtime_declaration()
        .expect("lane source posture is runtime-only")
        .source_lane();
    if actual == expected {
        WorthQueryIntentAdmissionSourceLaneEligibility::MatchesExpected(expected)
    } else {
        WorthQueryIntentAdmissionSourceLaneEligibility::Mismatch { expected, actual }
    }
}

fn lane_authority_posture(
    request: &WorthQueryRawIntentAdmissionRequest,
    expected: WorthQueryAuthorityLane,
) -> WorthQueryIntentAdmissionAuthorityLaneEligibility {
    let actual = request
        .runtime_declaration()
        .expect("lane authority posture is runtime-only")
        .target_lane();
    if actual == expected {
        WorthQueryIntentAdmissionAuthorityLaneEligibility::MatchesExpected(expected)
    } else {
        WorthQueryIntentAdmissionAuthorityLaneEligibility::Mismatch { expected, actual }
    }
}

fn capability_violation_for_denial(
    request: &WorthQueryRawIntentAdmissionRequest,
    stage: &'static str,
) -> WorthQueryIntentAdmissionCapabilityEligibility {
    let detail = match stage {
        "source-lane-admission" => match request
            .runtime_declaration()
            .expect("capability violation source lane is runtime-only")
            .source_lane()
        {
            WorthQueryIntentSourceLane::EffectTriggered => {
                "covered-runtime-entrypoint-rejects-effect-triggered-source-lane"
            }
            WorthQueryIntentSourceLane::PreviewLocal => {
                "covered-runtime-entrypoint-rejects-preview-local-source-lane"
            }
            WorthQueryIntentSourceLane::BranchLocal => {
                "covered-runtime-entrypoint-rejects-branch-local-source-lane"
            }
            WorthQueryIntentSourceLane::DerivedRuntime => {
                "covered-runtime-entrypoint-rejects-derived-runtime-source-lane"
            }
            WorthQueryIntentSourceLane::UserAuthored => {
                "declared-source-lane-is-not-admitted-for-this-covered-entrypoint"
            }
        },
        "authority-admission" => match request
            .runtime_declaration()
            .expect("capability violation authority lane is runtime-only")
            .target_lane()
        {
            WorthQueryAuthorityLane::BranchLocalTruth => {
                "covered-runtime-entrypoint-rejects-branch-local-truth-target"
            }
            WorthQueryAuthorityLane::PreviewTruth => {
                "covered-runtime-entrypoint-rejects-preview-truth-target"
            }
            WorthQueryAuthorityLane::DerivedRuntimeState => {
                "covered-runtime-entrypoint-rejects-derived-runtime-state-target"
            }
            WorthQueryAuthorityLane::EffectDeliveryState => {
                "covered-runtime-entrypoint-rejects-effect-delivery-state-target"
            }
            WorthQueryAuthorityLane::PendingWriteIntent => {
                "covered-runtime-entrypoint-rejects-pending-write-intent-target"
            }
            WorthQueryAuthorityLane::BridgeExternalState => {
                "covered-runtime-entrypoint-rejects-bridge-external-state-target"
            }
            WorthQueryAuthorityLane::TemporalExecutionState => {
                "covered-runtime-entrypoint-rejects-temporal-execution-state-target"
            }
            WorthQueryAuthorityLane::AsyncResourceState => {
                "covered-runtime-entrypoint-rejects-async-resource-state-target"
            }
            WorthQueryAuthorityLane::AuthoritativeTruth => {
                "declared-authority-lane-is-not-admitted-for-this-covered-entrypoint"
            }
            _ => "declared-authority-lane-is-not-admitted-for-this-covered-entrypoint",
        },
        _ => "runtime-intent-floor-capability-violation",
    };
    WorthQueryIntentAdmissionCapabilityEligibility::Violation { stage, detail }
}
