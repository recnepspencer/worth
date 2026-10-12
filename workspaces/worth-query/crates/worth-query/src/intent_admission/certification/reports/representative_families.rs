use crate::identity::hash_parts;
use crate::{
    basis_lifecycle::certify_basis_lifecycle,
    projection_consumption::certify_projection_consumption_closeout_core,
};

use super::super::fixtures::{
    certified_basis_observation_intent_fixture, certified_deferred_intent_fixture,
    certified_effect_intent_fixture, certified_inspection_advisory_redaction_fixture,
    certified_projection_consumption_admitted_fixture,
    certified_projection_consumption_warning_fixture, certified_routing_intent_fixture,
    certified_unsupported_intent_fixture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryIntentAdmissionRepresentativeFamilyLane {
    BasisParity,
    EffectAdmitted,
    ProjectionAdvisory,
    InspectionAdvisoryRedaction,
    RoutingAdmitted,
    RoutingFutureNeighbor,
}

impl WorthQueryIntentAdmissionRepresentativeFamilyLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BasisParity => "basis_parity",
            Self::EffectAdmitted => "effect_admitted",
            Self::ProjectionAdvisory => "projection_advisory",
            Self::InspectionAdvisoryRedaction => "inspection_advisory_redaction",
            Self::RoutingAdmitted => "routing_admitted",
            Self::RoutingFutureNeighbor => "routing_future_neighbor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionRepresentativeFamilyRow {
    lane: WorthQueryIntentAdmissionRepresentativeFamilyLane,
    authority_surface: &'static str,
    neighbor_certification_surface: &'static str,
    neighbor_bundle_digest: String,
    evidence_digest: String,
    posture_detail: &'static str,
    row_digest: String,
}

impl WorthQueryIntentAdmissionRepresentativeFamilyRow {
    pub fn lane(&self) -> WorthQueryIntentAdmissionRepresentativeFamilyLane {
        self.lane
    }

    pub fn authority_surface(&self) -> &'static str {
        self.authority_surface
    }

    pub fn neighbor_certification_surface(&self) -> &'static str {
        self.neighbor_certification_surface
    }

    pub fn neighbor_bundle_digest(&self) -> &str {
        &self.neighbor_bundle_digest
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }

    pub fn posture_detail(&self) -> &'static str {
        self.posture_detail
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionRepresentativeFamilyReport {
    rows: Vec<WorthQueryIntentAdmissionRepresentativeFamilyRow>,
    representative_family_coverage_digest: String,
}

impl WorthQueryIntentAdmissionRepresentativeFamilyReport {
    pub fn rows(&self) -> &[WorthQueryIntentAdmissionRepresentativeFamilyRow] {
        &self.rows
    }

    pub fn representative_family_coverage_digest(&self) -> &str {
        &self.representative_family_coverage_digest
    }
}

pub fn worth_query_intent_admission_representative_family_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionRepresentativeFamilyReport {
    let basis = certified_basis_observation_intent_fixture();
    let effect = certified_effect_intent_fixture(execution);
    let projection_admitted = certified_projection_consumption_admitted_fixture();
    let projection_warning = certified_projection_consumption_warning_fixture();
    let basis_bundle = certify_basis_lifecycle();
    let projection_bundle = certify_projection_consumption_closeout_core();
    let deferred = certified_deferred_intent_fixture();
    let unsupported = certified_unsupported_intent_fixture();
    let inspection = certified_inspection_advisory_redaction_fixture(execution);
    let routing = certified_routing_intent_fixture(execution);

    let rows = vec![
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::BasisParity,
            "intent_admission::worth_query_basis_observation_intent",
            "basis_lifecycle::certify_basis_lifecycle",
            basis_neighbor_bundle_digest(&basis_bundle),
            hash_parts(&[
                basis.request.request_digest().to_string(),
                basis.eligibility.eligibility_digest().to_string(),
                basis.plan.request_digest().to_string(),
                basis.scoped_basis_digest.clone(),
                basis_neighbor_bundle_digest(&basis_bundle),
            ]),
            "equivalent basis-use intents normalize through one intent-admission lattice path before scoping to lower-runtime evidence",
        ),
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::EffectAdmitted,
            "runtime.next_effect_write_intent(&effect, version, contract).review()?.admit()?.execute()",
            "effect-execution-covered-surface",
            hash_parts(&[
                effect.request_digest.clone(),
                effect.eligibility_digest.clone(),
                effect.decision_digest.clone(),
            ]),
            hash_parts(&[
                effect.request_digest.clone(),
                effect.eligibility_digest.clone(),
                effect.decision_digest.clone(),
                effect.handoff_digest.clone(),
                effect.binding_digest.clone(),
                effect.trace_digest.clone(),
                effect.receipt_digest.clone(),
                effect.execution_provenance_chain_digest.clone(),
            ]),
            "effect-triggered work resolves through one admitted lattice path and lowers into execution without rediscovering raw intent at the runtime seam",
        ),
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::ProjectionAdvisory,
            "intent_admission::worth_query_projection_consumption_intent",
            "projection_consumption::certify_projection_consumption_closeout_core",
            projection_neighbor_bundle_digest(&projection_bundle),
            hash_parts(&[
                projection_admitted.request.request_digest().to_string(),
                projection_admitted.eligibility.eligibility_digest().to_string(),
                projection_admitted.plan.request_digest().to_string(),
                projection_admitted.contract_digest.clone(),
                projection_warning.request.request_digest().to_string(),
                projection_warning.eligibility.eligibility_digest().to_string(),
                projection_warning.contract_digest.clone(),
                projection_warning
                    .plan
                    .warning_kinds()
                    .map(|warnings| warnings.warning_digest().to_string())
                    .unwrap_or_else(|| "no-warnings".to_string()),
                projection_neighbor_bundle_digest(&projection_bundle),
            ]),
            "warning-bearing and admitted projection consumption both resolve through one intent-admission lattice path before contract binding",
        ),
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::InspectionAdvisoryRedaction,
            "runtime.inspect_intent(target).review()?.admit()?.execute()",
            "inspection-materialization-covered-surface",
            hash_parts(&[
                inspection.boundary_audit_digest.clone(),
                inspection.full_artifact_digest.clone(),
                inspection.redacted_artifact_digest.clone(),
            ]),
            hash_parts(&[
                inspection.request_digest.clone(),
                inspection.eligibility_digest.clone(),
                inspection.decision_trace_digest.clone(),
                inspection.execution_provenance_chain_digest.clone(),
                inspection.full_artifact_digest.clone(),
                inspection.redacted_artifact_digest.clone(),
                inspection.causal_identity_digest.clone(),
                inspection.boundary_audit_digest.clone(),
            ]),
            "inspection detail narrowing changes materialized detail while preserving one causal identity on the lattice-owned unified inspection path",
        ),
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::RoutingAdmitted,
            "runtime.probe_existing_intent(request).review()?.admit()?.execute()",
            "routing-covered-surface",
            hash_parts(&[
                routing.request.request_digest().to_string(),
                decision_digest(&routing.decision),
                routing.handoff_digest.clone(),
            ]),
            hash_parts(&[
                routing.request.request_digest().to_string(),
                decision_digest(&routing.decision),
                routing.handoff_digest.clone(),
                routing.binding_digest.clone(),
                routing.trace_digest.clone(),
                routing.result.receipt().probe_digest().to_string(),
                routing
                    .result
                    .receipt()
                    .execution_provenance_chain_digest()
                    .unwrap_or("missing-routing-provenance")
                    .to_string(),
            ]),
            "existing-truth probe routing resolves through one admitted lattice path and executes with retained routing provenance instead of a convenience bypass",
        ),
        representative_row(
            WorthQueryIntentAdmissionRepresentativeFamilyLane::RoutingFutureNeighbor,
            "intent_admission::{admit_runtime_intent_request, WorthQueryRawIntentAdmissionRequest::deferred_neighbor}",
            "routing-future-neighbor-deferred-owner",
            hash_parts(&[
                deferred.trace.trace_digest().to_string(),
                unsupported.trace.trace_digest().to_string(),
            ]),
            hash_parts(&[
                deferred.request.request_digest().to_string(),
                decision_digest(&deferred.decision),
                deferred.trace.trace_digest().to_string(),
                unsupported.request.request_digest().to_string(),
                decision_digest(&unsupported.decision),
                unsupported.trace.trace_digest().to_string(),
            ]),
            "lower-runtime capability routing preserves a typed deferred lane and a distinct typed unsupported lane before 9.3.6 execution semantics",
        ),
    ];
    let representative_family_coverage_digest = hash_parts(
        &rows
            .iter()
            .map(|row| row.row_digest().to_string())
            .collect::<Vec<_>>(),
    );
    WorthQueryIntentAdmissionRepresentativeFamilyReport {
        rows,
        representative_family_coverage_digest,
    }
}

fn representative_row(
    lane: WorthQueryIntentAdmissionRepresentativeFamilyLane,
    authority_surface: &'static str,
    neighbor_certification_surface: &'static str,
    neighbor_bundle_digest: String,
    evidence_digest: String,
    posture_detail: &'static str,
) -> WorthQueryIntentAdmissionRepresentativeFamilyRow {
    let row_digest = hash_parts(&[
        "worth_query_intent_admission_representative_family_row_v1".to_string(),
        format!("lane:{}", lane.as_str()),
        format!("authority:{authority_surface}"),
        format!("neighbor_surface:{neighbor_certification_surface}"),
        format!("neighbor_bundle:{neighbor_bundle_digest}"),
        format!("evidence:{evidence_digest}"),
        format!("posture:{posture_detail}"),
    ]);
    WorthQueryIntentAdmissionRepresentativeFamilyRow {
        lane,
        authority_surface,
        neighbor_certification_surface,
        neighbor_bundle_digest,
        evidence_digest,
        posture_detail,
        row_digest,
    }
}

fn basis_neighbor_bundle_digest(
    bundle: &crate::basis_lifecycle::BasisLifecycleCertificationBundle,
) -> String {
    hash_parts(&[
        bundle
            .output_digest("basis_eligibility_digest")
            .expect("basis certification bundle should expose basis_eligibility_digest")
            .to_string(),
        bundle
            .output_digest("scoped_basis_digest")
            .expect("basis certification bundle should expose scoped_basis_digest")
            .to_string(),
        bundle
            .output_digest("basis_support_matrix_digest")
            .expect("basis certification bundle should expose basis_support_matrix_digest")
            .to_string(),
        bundle
            .output_digest("basis_target_dx_digest")
            .expect("basis certification bundle should expose basis_target_dx_digest")
            .to_string(),
    ])
}

fn projection_neighbor_bundle_digest(
    bundle: &crate::projection_consumption::ProjectionConsumptionCertificationBundle,
) -> String {
    hash_parts(&[
        bundle
            .output_digest("projection_consumption_eligibility_digest")
            .expect("projection certification bundle should expose projection_consumption_eligibility_digest")
            .to_string(),
        bundle
            .output_digest("materialized_projection_contract_digest")
            .expect(
                "projection certification bundle should expose materialized_projection_contract_digest",
            )
            .to_string(),
        bundle
            .output_digest("projection_support_matrix_digest")
            .expect(
                "projection certification bundle should expose projection_support_matrix_digest",
            )
            .to_string(),
        bundle
            .output_digest("projection_target_dx_digest")
            .expect("projection certification bundle should expose projection_target_dx_digest")
            .to_string(),
    ])
}

fn decision_digest(decision: &crate::facade::runtime::WorthQueryIntentAdmissionDecision) -> String {
    match decision {
        crate::facade::runtime::WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            plan.decision_digest().to_string()
        }
        crate::facade::runtime::WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
            advisory.decision_digest().to_string()
        }
        crate::facade::runtime::WorthQueryIntentAdmissionDecision::Violation(violation) => {
            violation.decision_digest().to_string()
        }
    }
}
