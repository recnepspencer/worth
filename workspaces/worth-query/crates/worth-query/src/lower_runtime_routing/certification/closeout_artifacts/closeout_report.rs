use crate::identity::hash_parts;

use super::{
    certify_lower_runtime_routing, worth_query_lower_runtime_closure_test,
    WorthQueryLowerRuntimeCertificationBundle, WorthQueryLowerRuntimeClosureTest,
};
use crate::lower_runtime_routing::certification::boundary_certification::{
    worth_query_lower_runtime_boundary_reconciliation_report,
    WorthQueryLowerRuntimeBoundaryReconciliationReport,
};
use crate::lower_runtime_routing::certification::phase_manifest::{
    worth_query_lower_runtime_phase_manifest, WorthQueryLowerRuntimePhaseManifest,
};
use crate::lower_runtime_routing::certification::surface::{
    worth_query_lower_runtime_acceptance_suite, worth_query_lower_runtime_synthetic_tail_report,
    WorthQueryLowerRuntimeAcceptanceLane, WorthQueryLowerRuntimeAcceptanceSuite,
    WorthQueryLowerRuntimeSyntheticTailReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCloseoutReport {
    certification_bundle: WorthQueryLowerRuntimeCertificationBundle,
    closure_test: WorthQueryLowerRuntimeClosureTest,
    phase_manifest: WorthQueryLowerRuntimePhaseManifest,
    acceptance_suite: WorthQueryLowerRuntimeAcceptanceSuite,
    boundary_reconciliation: WorthQueryLowerRuntimeBoundaryReconciliationReport,
    synthetic_tail_report: WorthQueryLowerRuntimeSyntheticTailReport,
    stabilization_target_digest: String,
    report_digest: String,
}

impl WorthQueryLowerRuntimeCloseoutReport {
    pub(crate) fn new(
        certification_bundle: WorthQueryLowerRuntimeCertificationBundle,
        closure_test: WorthQueryLowerRuntimeClosureTest,
        phase_manifest: WorthQueryLowerRuntimePhaseManifest,
        acceptance_suite: WorthQueryLowerRuntimeAcceptanceSuite,
        boundary_reconciliation: WorthQueryLowerRuntimeBoundaryReconciliationReport,
        synthetic_tail_report: WorthQueryLowerRuntimeSyntheticTailReport,
    ) -> Self {
        let stabilization_target_digest = hash_parts(&[
            certification_bundle
                .certification_bundle_digest()
                .to_string(),
            closure_test.suite_digest().to_string(),
            phase_manifest.manifest_digest().to_string(),
            phase_manifest.typestate_transition_digest().to_string(),
            acceptance_suite.suite_digest().to_string(),
            boundary_reconciliation.report_digest().to_string(),
            synthetic_tail_report.report_digest().to_string(),
        ]);
        let report_digest = hash_parts(&[
            "lower_runtime_routing_closeout_report_v1".to_string(),
            format!(
                "bundle:{}",
                certification_bundle.certification_bundle_digest()
            ),
            format!("closure_test:{}", closure_test.suite_digest()),
            format!("manifest:{}", phase_manifest.manifest_digest()),
            format!("typestate:{}", phase_manifest.typestate_transition_digest()),
            format!("acceptance:{}", acceptance_suite.suite_digest()),
            format!("reconciliation:{}", boundary_reconciliation.report_digest()),
            format!("synthetic_tail:{}", synthetic_tail_report.report_digest()),
            format!("stabilization:{stabilization_target_digest}"),
        ]);
        Self {
            certification_bundle,
            closure_test,
            phase_manifest,
            acceptance_suite,
            boundary_reconciliation,
            synthetic_tail_report,
            stabilization_target_digest,
            report_digest,
        }
    }

    pub fn certification_bundle(&self) -> &WorthQueryLowerRuntimeCertificationBundle {
        &self.certification_bundle
    }

    pub fn closure_test(&self) -> &WorthQueryLowerRuntimeClosureTest {
        &self.closure_test
    }

    pub fn phase_manifest(&self) -> &WorthQueryLowerRuntimePhaseManifest {
        &self.phase_manifest
    }

    pub fn acceptance_suite(&self) -> &WorthQueryLowerRuntimeAcceptanceSuite {
        &self.acceptance_suite
    }

    pub fn boundary_reconciliation(&self) -> &WorthQueryLowerRuntimeBoundaryReconciliationReport {
        &self.boundary_reconciliation
    }

    pub fn synthetic_tail_report(&self) -> &WorthQueryLowerRuntimeSyntheticTailReport {
        &self.synthetic_tail_report
    }

    pub fn stabilization_target_digest(&self) -> &str {
        &self.stabilization_target_digest
    }

    pub fn report_digest(&self) -> &str {
        &self.report_digest
    }
}

pub fn worth_query_lower_runtime_closeout_report(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeCloseoutReport {
    let certification_bundle = certify_lower_runtime_routing(resource_request);
    let closure_test = worth_query_lower_runtime_closure_test(resource_request);
    let phase_manifest = worth_query_lower_runtime_phase_manifest();
    let acceptance_suite = worth_query_lower_runtime_acceptance_suite(resource_request);
    let boundary_reconciliation = worth_query_lower_runtime_boundary_reconciliation_report();
    let synthetic_tail_report = worth_query_lower_runtime_synthetic_tail_report(resource_request);

    assert_eq!(
        closure_test
            .certification_bundle()
            .certification_bundle_digest(),
        certification_bundle.certification_bundle_digest(),
        "closeout report requires named closure test parity with the certification bundle"
    );
    assert_eq!(
        closure_test.acceptance_suite().suite_digest(),
        acceptance_suite.suite_digest(),
        "closeout report requires named closure test parity with the acceptance suite"
    );
    assert_eq!(
        closure_test.boundary_reconciliation().report_digest(),
        boundary_reconciliation.report_digest(),
        "closeout report requires named closure test parity with boundary reconciliation"
    );
    assert_eq!(
        certification_bundle.output_digest("route_phase_artifact_manifest_digest"),
        Some(phase_manifest.manifest_digest()),
        "closeout report requires certification bundle parity with the public phase manifest"
    );
    assert_eq!(
        certification_bundle.output_digest("route_typestate_transition_digest"),
        Some(phase_manifest.typestate_transition_digest()),
        "closeout report requires certification bundle parity with phase transitions"
    );
    assert_eq!(
        certification_bundle.output_digest("route_boundary_reconciliation_digest"),
        Some(boundary_reconciliation.report_digest()),
        "closeout report requires boundary reconciliation parity"
    );
    assert_eq!(
        closure_test.proof_shape_audit().proof_shape_digest(),
        certification_bundle.output_digest("route_proof_shape_digest").expect(
            "closeout report requires proof-shape output so the named closure test cannot drift"
        ),
        "closeout report requires named closure test parity with proof-shape audit"
    );
    assert_eq!(
        certification_bundle.output_digest("route_synthetic_tail_report_digest"),
        Some(synthetic_tail_report.report_digest()),
        "closeout report requires synthetic tail report parity"
    );
    assert_eq!(
        certification_bundle
            .rows()
            .iter()
            .find(|row| row.lane()
                == super::WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence)
            .map(|row| row.artifact_digest()),
        Some(acceptance_suite.suite_digest()),
        "closeout report requires acceptance evidence lane parity"
    );
    assert_eq!(
        acceptance_suite
            .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
            .digest(),
        certification_bundle
            .rows()
            .iter()
            .find(|row| row.lane()
                == super::WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence)
            .and_then(|row| row.failure_digest())
            .expect("acceptance evidence lane should expose hostile digest"),
        "closeout report requires hostile acceptance parity"
    );

    WorthQueryLowerRuntimeCloseoutReport::new(
        certification_bundle,
        closure_test,
        phase_manifest,
        acceptance_suite,
        boundary_reconciliation,
        synthetic_tail_report,
    )
}

pub fn worth_query_lower_runtime_closeout_report_digest(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    worth_query_lower_runtime_closeout_report(resource_request)
        .report_digest()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closeout_report_keeps_stabilization_inputs_in_sync() {
        let host_request =
            worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
                .serial_request(worth_execution::CancellationToken::new(), None);
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let report = worth_query_lower_runtime_closeout_report(resource_request);

        assert_eq!(
            report
                .closure_test()
                .certification_bundle()
                .certification_bundle_digest(),
            report.certification_bundle().certification_bundle_digest()
        );
        assert_eq!(
            report.closure_test().acceptance_suite().suite_digest(),
            report.acceptance_suite().suite_digest()
        );
        assert_eq!(
            report
                .certification_bundle()
                .output_digest("route_phase_artifact_manifest_digest"),
            Some(report.phase_manifest().manifest_digest())
        );
        assert_eq!(
            report
                .certification_bundle()
                .output_digest("route_typestate_transition_digest"),
            Some(report.phase_manifest().typestate_transition_digest())
        );
        assert_eq!(
            report
                .certification_bundle()
                .output_digest("route_boundary_reconciliation_digest"),
            Some(report.boundary_reconciliation().report_digest())
        );
        assert_eq!(
            report
                .closure_test()
                .boundary_reconciliation()
                .report_digest(),
            report.boundary_reconciliation().report_digest()
        );
        assert_eq!(
            report
                .closure_test()
                .proof_shape_audit()
                .proof_shape_digest(),
            report
                .certification_bundle()
                .output_digest("route_proof_shape_digest")
                .expect("proof-shape output should exist")
        );
        assert_eq!(
            report
                .certification_bundle()
                .output_digest("route_synthetic_tail_report_digest"),
            Some(report.synthetic_tail_report().report_digest())
        );
    }

    #[test]
    fn closeout_report_digest_is_distinct_from_bundle_digest() {
        let host_request =
            worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
                .serial_request(worth_execution::CancellationToken::new(), None);
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let report = worth_query_lower_runtime_closeout_report(resource_request);

        assert_eq!(
            worth_query_lower_runtime_closeout_report_digest(resource_request),
            report.report_digest()
        );
        assert_ne!(
            report.report_digest(),
            report.certification_bundle().certification_bundle_digest()
        );
        assert_ne!(
            report.stabilization_target_digest(),
            report.phase_manifest().manifest_digest()
        );
    }
}
