use crate::identity::hash_parts;

use super::super::fixtures::{
    certified_admitted_intent_fixture, certified_advisory_intent_fixture,
    certified_failure_intent_fixture, certified_read_intent_fixture,
    certified_violation_intent_fixture, legacy_delegation_parity_fixture,
};
use super::slope_runs::{
    lane_width_runs, slope_digest, WorthQueryIntentAdmissionSlopeLane,
    WorthQueryIntentAdmissionWidthRunRow,
};
use super::worth_query_intent_admission_legacy_parity_report;
use super::worth_query_intent_admission_representative_family_report;
use crate::intent_admission::{
    worth_query_intent_admission_coverage_inventory, worth_query_intent_admission_family_inventory,
    worth_query_intent_admission_support_matrix,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionCertificationCounterSnapshot {
    intent_family_lookup_width: usize,
    covered_entrypoint_lookup_width: usize,
    decision_trace_width: usize,
    execution_provenance_width: usize,
    digest: String,
}

impl WorthQueryIntentAdmissionCertificationCounterSnapshot {
    pub fn intent_family_lookup_width(&self) -> usize {
        self.intent_family_lookup_width
    }

    pub fn covered_entrypoint_lookup_width(&self) -> usize {
        self.covered_entrypoint_lookup_width
    }

    pub fn decision_trace_width(&self) -> usize {
        self.decision_trace_width
    }

    pub fn execution_provenance_width(&self) -> usize {
        self.execution_provenance_width
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionSlopeReport {
    counter_snapshot: WorthQueryIntentAdmissionCertificationCounterSnapshot,
    width_runs: Vec<WorthQueryIntentAdmissionWidthRunRow>,
    admission_classification_slope_digest: String,
    decision_trace_assembly_slope_digest: String,
    decision_support_lookup_slope_digest: String,
    covered_entrypoint_inventory_slope_digest: String,
    execution_provenance_assembly_slope_digest: String,
    legacy_delegation_parity_slope_digest: String,
    decision_certification_coverage_slope_digest: String,
}

impl WorthQueryIntentAdmissionSlopeReport {
    pub fn counter_snapshot(&self) -> &WorthQueryIntentAdmissionCertificationCounterSnapshot {
        &self.counter_snapshot
    }

    pub fn width_runs(&self) -> &[WorthQueryIntentAdmissionWidthRunRow] {
        &self.width_runs
    }

    pub fn admission_classification_slope_digest(&self) -> &str {
        &self.admission_classification_slope_digest
    }

    pub fn decision_trace_assembly_slope_digest(&self) -> &str {
        &self.decision_trace_assembly_slope_digest
    }

    pub fn decision_support_lookup_slope_digest(&self) -> &str {
        &self.decision_support_lookup_slope_digest
    }

    pub fn covered_entrypoint_inventory_slope_digest(&self) -> &str {
        &self.covered_entrypoint_inventory_slope_digest
    }

    pub fn execution_provenance_assembly_slope_digest(&self) -> &str {
        &self.execution_provenance_assembly_slope_digest
    }

    pub fn legacy_delegation_parity_slope_digest(&self) -> &str {
        &self.legacy_delegation_parity_slope_digest
    }

    pub fn decision_certification_coverage_slope_digest(&self) -> &str {
        &self.decision_certification_coverage_slope_digest
    }
}

pub fn worth_query_intent_admission_slope_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionSlopeReport {
    let admitted = certified_admitted_intent_fixture(execution);
    let advisory = certified_advisory_intent_fixture();
    let violation = certified_violation_intent_fixture();
    let failure = certified_failure_intent_fixture(execution);
    let read = certified_read_intent_fixture(execution);
    let family_inventory = worth_query_intent_admission_family_inventory();
    let coverage_inventory = worth_query_intent_admission_coverage_inventory();
    let support_matrix = worth_query_intent_admission_support_matrix();
    let parity_report = worth_query_intent_admission_legacy_parity_report(execution);
    let representative_family_report =
        worth_query_intent_admission_representative_family_report(execution);
    let parity_fixture = legacy_delegation_parity_fixture(execution);

    let decision_trace_width = [
        admitted.trace.rows().len(),
        advisory.trace.rows().len(),
        violation.trace.rows().len(),
        failure.trace.rows().len(),
        read.trace.rows().len(),
    ]
    .into_iter()
    .max()
    .expect("certified traces should exist");
    let execution_provenance_components = execution_provenance_components(
        admitted.receipt.execution_provenance_chain_digest(),
        &failure.execution_provenance_chain_digest,
        read.result
            .receipt()
            .execution_provenance_chain_digest()
            .unwrap_or("missing-read-provenance"),
        admitted.binding.binding_digest(),
        read.binding.binding_digest(),
    );
    let counter_snapshot = WorthQueryIntentAdmissionCertificationCounterSnapshot {
        intent_family_lookup_width: family_inventory.rows().len(),
        covered_entrypoint_lookup_width: coverage_inventory.rows().len(),
        decision_trace_width,
        execution_provenance_width: execution_provenance_components.len(),
        digest: hash_parts(&[
            "worth_query_intent_admission_counter_snapshot_v2".to_string(),
            format!(
                "intent_family_lookup_width:{}",
                family_inventory.rows().len()
            ),
            format!(
                "covered_entrypoint_lookup_width:{}",
                coverage_inventory.rows().len()
            ),
            format!("decision_trace_width:{decision_trace_width}"),
            format!(
                "execution_provenance_width:{}",
                execution_provenance_components.len()
            ),
        ]),
    };

    let width_runs = [
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::AdmissionClassification,
            family_inventory
                .rows()
                .iter()
                .map(|row| row.family().as_str().to_string())
                .collect(),
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::DecisionTraceAssembly,
            admitted
                .trace
                .rows()
                .iter()
                .chain(advisory.trace.rows().iter())
                .chain(violation.trace.rows().iter())
                .chain(failure.trace.rows().iter())
                .chain(read.trace.rows().iter())
                .map(|row| row.row_digest().to_string())
                .collect(),
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::DecisionSupportLookup,
            support_matrix
                .rows()
                .iter()
                .map(|row| {
                    format!(
                        "{}:{}:{}",
                        row.family().as_str(),
                        row.entrypoint().as_str(),
                        row.detail().as_str()
                    )
                })
                .collect(),
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::CoveredEntrypointInventory,
            coverage_inventory
                .rows()
                .iter()
                .map(|row| {
                    format!(
                        "{}:{}:{}",
                        row.family().as_str(),
                        row.entrypoint().as_str(),
                        row.execution_boundary().as_str()
                    )
                })
                .collect(),
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::ExecutionProvenanceAssembly,
            execution_provenance_components,
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::LegacyDelegationParity,
            parity_report
                .rows()
                .iter()
                .zip([
                    parity_fixture
                        .authoritative_legacy
                        .receipt_digest()
                        .to_string(),
                    parity_fixture
                        .effect_legacy
                        .intent_receipt()
                        .receipt_digest()
                        .to_string(),
                    parity_fixture
                        .read_current_legacy
                        .receipt()
                        .result_digest()
                        .to_string(),
                    parity_fixture
                        .read_basis_legacy
                        .receipt()
                        .result_digest()
                        .to_string(),
                ])
                .map(|(row, result_digest)| format!("{}:{result_digest}", row.row_digest()))
                .collect(),
        ),
        lane_width_runs(
            WorthQueryIntentAdmissionSlopeLane::DecisionCertificationCoverage,
            family_inventory
                .rows()
                .iter()
                .map(|row| format!("family:{}", row.family().as_str()))
                .chain(
                    coverage_inventory
                        .rows()
                        .iter()
                        .map(|row| format!("coverage:{}", row.entrypoint().as_str())),
                )
                .chain(
                    support_matrix
                        .rows()
                        .iter()
                        .map(|row| format!("support:{}", row.entrypoint().as_str())),
                )
                .chain(
                    parity_report
                        .rows()
                        .iter()
                        .map(|row| format!("parity:{}", row.row_digest())),
                )
                .chain(
                    representative_family_report
                        .rows()
                        .iter()
                        .map(|row| format!("representative:{}", row.row_digest())),
                )
                .collect(),
        ),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    WorthQueryIntentAdmissionSlopeReport {
        counter_snapshot,
        admission_classification_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::AdmissionClassification,
        ),
        decision_trace_assembly_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::DecisionTraceAssembly,
        ),
        decision_support_lookup_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::DecisionSupportLookup,
        ),
        covered_entrypoint_inventory_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::CoveredEntrypointInventory,
        ),
        execution_provenance_assembly_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::ExecutionProvenanceAssembly,
        ),
        legacy_delegation_parity_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::LegacyDelegationParity,
        ),
        decision_certification_coverage_slope_digest: slope_digest(
            &width_runs,
            WorthQueryIntentAdmissionSlopeLane::DecisionCertificationCoverage,
        ),
        width_runs,
    }
}

fn execution_provenance_components(
    admitted_receipt_digest: &str,
    failure_digest: &str,
    read_digest: &str,
    admitted_binding_digest: &str,
    read_binding_digest: &str,
) -> Vec<String> {
    vec![
        admitted_receipt_digest.to_string(),
        failure_digest.to_string(),
        read_digest.to_string(),
        admitted_binding_digest.to_string(),
        read_binding_digest.to_string(),
    ]
}
