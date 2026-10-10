mod outputs;

use self::outputs::{assemble_certification_outputs, certification_bundle_digest};
use super::audits::{
    worth_query_intent_admission_proof_shape_audit, WorthQueryIntentAdmissionProofShapeAudit,
    WorthQueryIntentAdmissionPublicBoundaryAudit, WorthQueryIntentAdmissionTopologyAudit,
};
use super::oracles::{
    worth_query_intent_admission_oracle_report, WorthQueryIntentAdmissionOracleReport,
};
use super::output_manifest::worth_query_intent_admission_certification_output_manifest;
use super::reports::{
    worth_query_intent_admission_legacy_parity_report,
    worth_query_intent_admission_representative_family_report,
    worth_query_intent_admission_representative_output_report,
    worth_query_intent_admission_seeded_certification_report,
    worth_query_intent_admission_slope_report,
    worth_query_intent_admission_support_traceability_report,
    WorthQueryIntentAdmissionCertificationCounterSnapshot,
    WorthQueryIntentAdmissionLegacyParityReport,
    WorthQueryIntentAdmissionRepresentativeFamilyReport,
    WorthQueryIntentAdmissionRepresentativeOutputReport,
    WorthQueryIntentAdmissionSeededCertificationReport, WorthQueryIntentAdmissionSlopeReport,
    WorthQueryIntentAdmissionSupportTraceabilityReport,
};
use crate::intent_admission::{
    worth_query_intent_admission_coverage_inventory, worth_query_intent_admission_family_inventory,
    worth_query_intent_admission_support_matrix, WorthQueryIntentAdmissionCoverageInventory,
    WorthQueryIntentAdmissionFamilyInventory, WorthQueryIntentAdmissionSupportMatrix,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionCertificationOutput {
    name: &'static str,
    digest: String,
}

impl WorthQueryIntentAdmissionCertificationOutput {
    pub(crate) fn new(name: &'static str, digest: String) -> Self {
        Self { name, digest }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionCertificationBundle {
    output_manifest: Vec<&'static str>,
    family_inventory: WorthQueryIntentAdmissionFamilyInventory,
    coverage_inventory: WorthQueryIntentAdmissionCoverageInventory,
    support_matrix: WorthQueryIntentAdmissionSupportMatrix,
    public_boundary_audit: WorthQueryIntentAdmissionPublicBoundaryAudit,
    proof_shape_audit: WorthQueryIntentAdmissionProofShapeAudit,
    topology_audit: WorthQueryIntentAdmissionTopologyAudit,
    representative_output_report: WorthQueryIntentAdmissionRepresentativeOutputReport,
    representative_family_report: WorthQueryIntentAdmissionRepresentativeFamilyReport,
    oracle_report: WorthQueryIntentAdmissionOracleReport,
    legacy_parity_report: WorthQueryIntentAdmissionLegacyParityReport,
    support_traceability_report: WorthQueryIntentAdmissionSupportTraceabilityReport,
    seeded_report: WorthQueryIntentAdmissionSeededCertificationReport,
    slope_report: WorthQueryIntentAdmissionSlopeReport,
    outputs: Vec<WorthQueryIntentAdmissionCertificationOutput>,
    certification_bundle_digest: String,
}

impl WorthQueryIntentAdmissionCertificationBundle {
    fn new(
        family_inventory: WorthQueryIntentAdmissionFamilyInventory,
        coverage_inventory: WorthQueryIntentAdmissionCoverageInventory,
        support_matrix: WorthQueryIntentAdmissionSupportMatrix,
        public_boundary_audit: WorthQueryIntentAdmissionPublicBoundaryAudit,
        proof_shape_audit: WorthQueryIntentAdmissionProofShapeAudit,
        topology_audit: WorthQueryIntentAdmissionTopologyAudit,
        representative_output_report: WorthQueryIntentAdmissionRepresentativeOutputReport,
        representative_family_report: WorthQueryIntentAdmissionRepresentativeFamilyReport,
        oracle_report: WorthQueryIntentAdmissionOracleReport,
        legacy_parity_report: WorthQueryIntentAdmissionLegacyParityReport,
        support_traceability_report: WorthQueryIntentAdmissionSupportTraceabilityReport,
        seeded_report: WorthQueryIntentAdmissionSeededCertificationReport,
        slope_report: WorthQueryIntentAdmissionSlopeReport,
    ) -> Self {
        let output_manifest = worth_query_intent_admission_certification_output_manifest().to_vec();
        let output_specs = assemble_certification_outputs(
            &family_inventory,
            &coverage_inventory,
            &support_matrix,
            &public_boundary_audit,
            &proof_shape_audit,
            &topology_audit,
            &representative_output_report,
            &representative_family_report,
            &oracle_report,
            &legacy_parity_report,
            &support_traceability_report,
            &seeded_report,
            &slope_report,
        );
        validate_output_manifest(&output_manifest, &output_specs);
        let certification_bundle_digest = certification_bundle_digest(&output_specs);
        let outputs = output_specs
            .iter()
            .map(|output| {
                WorthQueryIntentAdmissionCertificationOutput::new(
                    output.name(),
                    output.digest().to_string(),
                )
            })
            .collect::<Vec<_>>();
        Self {
            output_manifest,
            family_inventory,
            coverage_inventory,
            support_matrix,
            public_boundary_audit,
            proof_shape_audit,
            topology_audit,
            representative_output_report,
            representative_family_report,
            oracle_report,
            legacy_parity_report,
            support_traceability_report,
            seeded_report,
            slope_report,
            outputs,
            certification_bundle_digest,
        }
    }

    pub fn output_manifest(&self) -> &[&'static str] {
        &self.output_manifest
    }

    pub fn family_inventory(&self) -> &WorthQueryIntentAdmissionFamilyInventory {
        &self.family_inventory
    }

    pub fn coverage_inventory(&self) -> &WorthQueryIntentAdmissionCoverageInventory {
        &self.coverage_inventory
    }

    pub fn support_matrix(&self) -> &WorthQueryIntentAdmissionSupportMatrix {
        &self.support_matrix
    }

    pub fn public_boundary_audit(&self) -> &WorthQueryIntentAdmissionPublicBoundaryAudit {
        &self.public_boundary_audit
    }

    pub fn proof_shape_audit(&self) -> &WorthQueryIntentAdmissionProofShapeAudit {
        &self.proof_shape_audit
    }

    pub fn topology_audit(&self) -> &WorthQueryIntentAdmissionTopologyAudit {
        &self.topology_audit
    }

    pub fn representative_output_report(
        &self,
    ) -> &WorthQueryIntentAdmissionRepresentativeOutputReport {
        &self.representative_output_report
    }

    pub fn representative_family_report(
        &self,
    ) -> &WorthQueryIntentAdmissionRepresentativeFamilyReport {
        &self.representative_family_report
    }

    pub fn oracle_report(&self) -> &WorthQueryIntentAdmissionOracleReport {
        &self.oracle_report
    }

    pub fn legacy_parity_report(&self) -> &WorthQueryIntentAdmissionLegacyParityReport {
        &self.legacy_parity_report
    }

    pub fn support_traceability_report(
        &self,
    ) -> &WorthQueryIntentAdmissionSupportTraceabilityReport {
        &self.support_traceability_report
    }

    pub fn seeded_report(&self) -> &WorthQueryIntentAdmissionSeededCertificationReport {
        &self.seeded_report
    }

    pub fn counter_snapshot(&self) -> &WorthQueryIntentAdmissionCertificationCounterSnapshot {
        self.slope_report.counter_snapshot()
    }

    pub fn slope_report(&self) -> &WorthQueryIntentAdmissionSlopeReport {
        &self.slope_report
    }

    pub fn outputs(&self) -> &[WorthQueryIntentAdmissionCertificationOutput] {
        &self.outputs
    }

    pub fn output_digest(&self, key: &str) -> Option<&str> {
        self.outputs
            .iter()
            .find(|output| output.name() == key)
            .map(WorthQueryIntentAdmissionCertificationOutput::digest)
    }

    pub fn certification_bundle_digest(&self) -> &str {
        &self.certification_bundle_digest
    }
}

pub fn certify_intent_admission(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionCertificationBundle {
    WorthQueryIntentAdmissionCertificationBundle::new(
        worth_query_intent_admission_family_inventory(),
        worth_query_intent_admission_coverage_inventory(),
        worth_query_intent_admission_support_matrix(),
        WorthQueryIntentAdmissionPublicBoundaryAudit::new(),
        worth_query_intent_admission_proof_shape_audit(),
        WorthQueryIntentAdmissionTopologyAudit::new(),
        worth_query_intent_admission_representative_output_report(execution),
        worth_query_intent_admission_representative_family_report(execution),
        worth_query_intent_admission_oracle_report(execution),
        worth_query_intent_admission_legacy_parity_report(execution),
        worth_query_intent_admission_support_traceability_report(execution),
        worth_query_intent_admission_seeded_certification_report(execution),
        worth_query_intent_admission_slope_report(execution),
    )
}

fn validate_output_manifest(
    output_manifest: &[&'static str],
    outputs: &[self::outputs::WorthQueryIntentAdmissionCertificationOutputSpec],
) {
    let actual_names = outputs
        .iter()
        .map(|output| output.name())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_names.len(),
        actual_names
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        "intent-admission certification outputs must be duplicate-free"
    );
    assert_eq!(
        actual_names, output_manifest,
        "intent-admission certification outputs must match the compile-visible manifest exactly"
    );
}
