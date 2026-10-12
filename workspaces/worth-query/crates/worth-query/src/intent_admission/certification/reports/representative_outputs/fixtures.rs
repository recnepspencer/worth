use crate::facade::runtime::WorthQueryIntentAdmissionDecision;
use crate::identity::hash_parts;
use crate::intent_admission::{
    WorthQueryAdmittedIntentPlan, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryIntentDecisionTraceEvidence, WorthQueryIntentDecisionTraceRow,
    WorthQueryIntentEligibilityTraceEvidence,
};

use super::super::super::fixtures::{
    certified_admitted_intent_fixture, certified_advisory_intent_fixture,
    certified_basis_observation_intent_fixture, certified_failure_intent_fixture,
    certified_projection_consumption_admitted_fixture,
    certified_projection_consumption_warning_fixture, certified_read_intent_fixture,
    certified_routing_intent_fixture, certified_violation_intent_fixture,
    CertifiedAdmittedIntentFixture, CertifiedAdvisoryIntentFixture, CertifiedFailureIntentFixture,
    CertifiedReadIntentFixture, CertifiedRoutingIntentFixture, CertifiedViolationIntentFixture,
};

pub(super) type OutputRow = (&'static str, String);

#[derive(Clone)]
pub(super) struct RepresentativeOutputFixtures {
    pub(super) admitted: CertifiedAdmittedIntentFixture,
    pub(super) advisory: CertifiedAdvisoryIntentFixture,
    pub(super) violation: CertifiedViolationIntentFixture,
    pub(super) failure: CertifiedFailureIntentFixture,
    pub(super) read: CertifiedReadIntentFixture,
    pub(super) routing: CertifiedRoutingIntentFixture,
}

#[derive(Clone)]
pub(super) struct RepresentativeEligibilityDigests {
    pub(super) admitted: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) advisory: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) violation: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) failure: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) read: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) routing: WorthQueryIntentEligibilityTraceEvidence,
}

impl RepresentativeOutputFixtures {
    pub(super) fn load(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        Self {
            admitted: certified_admitted_intent_fixture(execution),
            advisory: certified_advisory_intent_fixture(),
            violation: certified_violation_intent_fixture(),
            failure: certified_failure_intent_fixture(execution),
            read: certified_read_intent_fixture(execution),
            routing: certified_routing_intent_fixture(execution),
        }
    }

    pub(super) fn eligibility(&self) -> RepresentativeEligibilityDigests {
        RepresentativeEligibilityDigests {
            admitted: eligibility_evidence(&self.admitted.trace).clone(),
            advisory: eligibility_evidence(&self.advisory.trace).clone(),
            violation: eligibility_evidence(&self.violation.trace).clone(),
            failure: eligibility_evidence(&self.failure.trace).clone(),
            read: eligibility_evidence(&self.read.trace).clone(),
            routing: eligibility_evidence(
                self.routing
                    .result
                    .receipt()
                    .decision_trace_envelope()
                    .expect("routing representative fixture should retain a trace"),
            )
            .clone(),
        }
    }
}

pub(super) fn basis_observation_fixture_digest_output() -> OutputRow {
    let basis = certified_basis_observation_intent_fixture();
    output(
        "basis_observation_fixture_digest",
        hash_parts(&[
            basis.request.request_digest().to_string(),
            basis.eligibility.eligibility_digest().to_string(),
            basis.plan.request_digest().to_string(),
            basis.scoped_basis_digest.clone(),
        ]),
    )
}

pub(super) fn projection_consumption_fixture_digest_output() -> OutputRow {
    let admitted = certified_projection_consumption_admitted_fixture();
    let warning = certified_projection_consumption_warning_fixture();
    output(
        "projection_consumption_fixture_digest",
        hash_parts(&[
            admitted.request.request_digest().to_string(),
            admitted.eligibility.eligibility_digest().to_string(),
            admitted.plan.request_digest().to_string(),
            admitted.contract_digest.clone(),
            warning.request.request_digest().to_string(),
            warning.eligibility.eligibility_digest().to_string(),
            warning.plan.request_digest().to_string(),
            warning.contract_digest.clone(),
            warning
                .plan
                .warning_kinds()
                .map(|warnings| warnings.warning_digest().to_string())
                .unwrap_or_else(|| "no-warnings".to_string()),
        ]),
    )
}

pub(super) fn decision_digest(decision: &WorthQueryIntentAdmissionDecision) -> String {
    match decision {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => plan.decision_digest().to_string(),
        WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
            advisory.decision_digest().to_string()
        }
        WorthQueryIntentAdmissionDecision::Violation(violation) => {
            violation.decision_digest().to_string()
        }
    }
}

pub(super) fn admitted_plan_digest(plan: &WorthQueryAdmittedIntentPlan) -> String {
    hash_parts(&[
        "worth_query_intent_admission_representative_plan_v1".to_string(),
        format!("family:{}", plan.family().as_str()),
        format!("entrypoint:{}", plan.entrypoint().as_str()),
        format!(
            "execution_seam:{}",
            plan.execution_seam()
                .map(|seam| seam.as_str())
                .unwrap_or("no-execution-handoff")
        ),
        format!("request:{}", plan.request_digest()),
        format!("eligibility:{}", plan.eligibility_digest()),
        format!("decision:{}", plan.decision_digest()),
    ])
}

pub(super) fn trace_row_digest(traces: &[&WorthQueryIntentDecisionTraceEnvelope]) -> String {
    hash_parts(
        &traces
            .iter()
            .flat_map(|trace| {
                trace
                    .rows()
                    .iter()
                    .map(WorthQueryIntentDecisionTraceRow::row_digest)
            })
            .map(str::to_string)
            .collect::<Vec<_>>(),
    )
}

pub(super) fn posture_digest<T: std::fmt::Debug, const N: usize>(values: [T; N]) -> String {
    hash_parts(&values.map(|value| format!("{value:?}")))
}

pub(super) fn digest_output_rows(outputs: &[OutputRow]) -> String {
    hash_parts(
        &outputs
            .iter()
            .map(|(name, digest)| format!("{name}:{digest}"))
            .collect::<Vec<_>>(),
    )
}

pub(super) fn output(name: &'static str, digest: String) -> OutputRow {
    (name, digest)
}

fn eligibility_evidence(
    trace: &WorthQueryIntentDecisionTraceEnvelope,
) -> &WorthQueryIntentEligibilityTraceEvidence {
    trace
        .rows()
        .iter()
        .find_map(|row| match row.evidence() {
            WorthQueryIntentDecisionTraceEvidence::Eligibility(evidence) => Some(evidence),
            _ => None,
        })
        .expect("representative trace should preserve eligibility evidence")
}
