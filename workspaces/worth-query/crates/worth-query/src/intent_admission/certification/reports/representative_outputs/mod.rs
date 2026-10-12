mod fixtures;

use crate::intent_admission::WorthQueryAdmittedIntentPlan;

use fixtures::{
    admitted_plan_digest, basis_observation_fixture_digest_output, decision_digest,
    digest_output_rows, output, posture_digest, projection_consumption_fixture_digest_output,
    trace_row_digest, OutputRow, RepresentativeEligibilityDigests, RepresentativeOutputFixtures,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionRepresentativeOutputReport {
    outputs: Vec<OutputRow>,
    report_digest: String,
}

impl WorthQueryIntentAdmissionRepresentativeOutputReport {
    pub fn digest_for(&self, name: &str) -> Option<&str> {
        self.outputs
            .iter()
            .find(|(output_name, _)| *output_name == name)
            .map(|(_, digest)| digest.as_str())
    }

    pub fn report_digest(&self) -> &str {
        &self.report_digest
    }
}

pub fn worth_query_intent_admission_representative_output_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionRepresentativeOutputReport {
    let fixtures = RepresentativeOutputFixtures::load(execution);
    let eligibility = fixtures.eligibility();
    let outputs = representative_outputs(&fixtures, &eligibility);
    let report_digest = digest_output_rows(&outputs);
    WorthQueryIntentAdmissionRepresentativeOutputReport {
        outputs,
        report_digest,
    }
}

fn representative_outputs(
    fixtures: &RepresentativeOutputFixtures,
    eligibility: &RepresentativeEligibilityDigests,
) -> Vec<OutputRow> {
    vec![
        query_digest_output(fixtures),
        raw_intent_digest_output(fixtures),
        intent_eligibility_digest_output(eligibility),
        admission_decision_digest_output(fixtures),
        admitted_intent_plan_digest_output(fixtures),
        admitted_execution_handoff_digest_output(fixtures),
        advisory_decision_digest_output(fixtures),
        violation_decision_digest_output(fixtures),
        decision_trace_digest_output(fixtures),
        decision_trace_envelope_digest_output(fixtures),
        posture_output(
            "policy_decision_digest",
            [
                eligibility.admitted.policy_posture(),
                eligibility.advisory.policy_posture(),
                eligibility.violation.policy_posture(),
                eligibility.failure.policy_posture(),
                eligibility.read.policy_posture(),
            ],
        ),
        posture_output(
            "capability_decision_digest",
            [
                eligibility.admitted.capability_posture(),
                eligibility.advisory.capability_posture(),
                eligibility.violation.capability_posture(),
                eligibility.failure.capability_posture(),
                eligibility.read.capability_posture(),
            ],
        ),
        posture_output(
            "invariant_decision_digest",
            [
                eligibility.admitted.invariant_posture(),
                eligibility.advisory.invariant_posture(),
                eligibility.violation.invariant_posture(),
                eligibility.failure.invariant_posture(),
                eligibility.read.invariant_posture(),
            ],
        ),
        posture_output(
            "basis_decision_digest",
            [
                eligibility.admitted.basis_posture(),
                eligibility.advisory.basis_posture(),
                eligibility.violation.basis_posture(),
                eligibility.failure.basis_posture(),
                eligibility.read.basis_posture(),
            ],
        ),
        posture_output(
            "projection_decision_digest",
            [
                eligibility.admitted.projection_source_posture(),
                eligibility.advisory.projection_source_posture(),
                eligibility.violation.projection_source_posture(),
                eligibility.failure.projection_source_posture(),
                eligibility.read.projection_source_posture(),
            ],
        ),
        posture_output(
            "routing_posture_digest",
            [
                eligibility.admitted.routing_support_posture(),
                eligibility.advisory.routing_support_posture(),
                eligibility.violation.routing_support_posture(),
                eligibility.failure.routing_support_posture(),
                eligibility.read.routing_support_posture(),
            ],
        ),
        execution_provenance_chain_digest_output(fixtures),
        output("failure_digest", fixtures.failure.failure_digest.clone()),
        basis_observation_fixture_digest_output(),
        projection_consumption_fixture_digest_output(),
    ]
}

fn query_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "query_digest",
        crate::identity::hash_parts(&[
            fixtures.admitted.receipt.receipt_digest().to_string(),
            fixtures.advisory.trace.trace_digest().to_string(),
            fixtures.violation.trace.trace_digest().to_string(),
            fixtures.failure.failure_digest.clone(),
            fixtures.read.result.receipt().result_digest().to_string(),
            fixtures.routing.result.receipt().probe_digest().to_string(),
        ]),
    )
}

fn raw_intent_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "raw_intent_digest",
        crate::identity::hash_parts(&[
            fixtures.admitted.request.request_digest().to_string(),
            fixtures.advisory.request.request_digest().to_string(),
            fixtures.violation.request.request_digest().to_string(),
            fixtures.failure.request.request_digest().to_string(),
            fixtures.read.request.request_digest().to_string(),
            fixtures.routing.request.request_digest().to_string(),
        ]),
    )
}

fn intent_eligibility_digest_output(eligibility: &RepresentativeEligibilityDigests) -> OutputRow {
    output(
        "intent_eligibility_digest",
        crate::identity::hash_parts(&[
            eligibility.admitted.eligibility_digest().to_string(),
            eligibility.advisory.eligibility_digest().to_string(),
            eligibility.violation.eligibility_digest().to_string(),
            eligibility.failure.eligibility_digest().to_string(),
            eligibility.read.eligibility_digest().to_string(),
            eligibility.routing.eligibility_digest().to_string(),
        ]),
    )
}

fn admission_decision_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "admission_decision_digest",
        crate::identity::hash_parts(&[
            fixtures.admitted.plan.decision_digest().to_string(),
            fixtures.read.plan.decision_digest().to_string(),
            decision_digest(&fixtures.routing.decision),
        ]),
    )
}

fn admitted_intent_plan_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "admitted_intent_plan_digest",
        crate::identity::hash_parts(&[
            admitted_plan_digest(&fixtures.admitted.plan),
            admitted_plan_digest(&WorthQueryAdmittedIntentPlan::ReadExecution(
                fixtures.read.plan.clone(),
            )),
            match &fixtures.routing.decision {
                crate::facade::runtime::WorthQueryIntentAdmissionDecision::Admitted(plan) => {
                    admitted_plan_digest(plan)
                }
                other => panic!("routing representative fixture should admit, got {other:?}"),
            },
        ]),
    )
}

fn admitted_execution_handoff_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "admitted_execution_handoff_digest",
        crate::identity::hash_parts(&[
            fixtures.admitted.handoff.handoff_digest().to_string(),
            fixtures.read.handoff.handoff_digest().to_string(),
            fixtures.routing.handoff_digest.clone(),
        ]),
    )
}

fn advisory_decision_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "advisory_decision_digest",
        decision_digest(&fixtures.advisory.decision),
    )
}

fn violation_decision_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "violation_decision_digest",
        decision_digest(&fixtures.violation.decision),
    )
}

fn decision_trace_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "decision_trace_digest",
        trace_row_digest(&[
            &fixtures.admitted.trace,
            &fixtures.advisory.trace,
            &fixtures.violation.trace,
            &fixtures.failure.trace,
        ]),
    )
}

fn decision_trace_envelope_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "decision_trace_envelope_digest",
        crate::identity::hash_parts(&[
            fixtures.admitted.trace.trace_digest().to_string(),
            fixtures.advisory.trace.trace_digest().to_string(),
            fixtures.violation.trace.trace_digest().to_string(),
            fixtures.failure.trace.trace_digest().to_string(),
            fixtures.read.trace.trace_digest().to_string(),
            fixtures.routing.trace_digest.clone(),
        ]),
    )
}

fn execution_provenance_chain_digest_output(fixtures: &RepresentativeOutputFixtures) -> OutputRow {
    output(
        "execution_provenance_chain_digest",
        crate::identity::hash_parts(&[
            fixtures
                .admitted
                .receipt
                .execution_provenance_chain_digest()
                .to_string(),
            fixtures.failure.execution_provenance_chain_digest.clone(),
            fixtures.admitted.binding.binding_digest().to_string(),
            fixtures
                .read
                .result
                .receipt()
                .execution_provenance_chain_digest()
                .unwrap_or("no-read-provenance")
                .to_string(),
            fixtures.read.binding.binding_digest().to_string(),
            fixtures
                .routing
                .result
                .receipt()
                .execution_provenance_chain_digest()
                .unwrap_or("no-routing-provenance")
                .to_string(),
            fixtures.routing.binding_digest.clone(),
        ]),
    )
}

fn posture_output<T: std::fmt::Debug, const N: usize>(
    name: &'static str,
    values: [T; N],
) -> OutputRow {
    output(name, posture_digest(values))
}
