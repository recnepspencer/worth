use crate::identity::hash_parts;

use super::{
    certify_lower_runtime_routing, WorthQueryLowerRuntimeCertificationBundle,
    WorthQueryLowerRuntimeCertificationLane,
};
use crate::lower_runtime_routing::certification::boundary_certification::{
    worth_query_lower_runtime_boundary_reconciliation_report,
    worth_query_lower_runtime_proof_shape_audit,
    WorthQueryLowerRuntimeBoundaryReconciliationReport, WorthQueryLowerRuntimeProofShapeAudit,
};
use crate::lower_runtime_routing::certification::surface::{
    worth_query_lower_runtime_acceptance_suite, WorthQueryLowerRuntimeAcceptanceLane,
    WorthQueryLowerRuntimeAcceptanceSuite,
};

pub const LOWER_RUNTIME_CLOSURE_TEST_NAME: &str =
    "9.3.6. Lower-Runtime Capability Routing And Boundary Envelope Closure Test";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeClosureTestLane {
    Control,
    Hostile,
    Parity,
    DownstreamBoundary,
}

impl WorthQueryLowerRuntimeClosureTestLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control-lane",
            Self::Hostile => "hostile-lane",
            Self::Parity => "parity-lane",
            Self::DownstreamBoundary => "downstream-boundary-lane",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeClosureTestRow {
    lane: WorthQueryLowerRuntimeClosureTestLane,
    digest: String,
    detail: String,
}

impl WorthQueryLowerRuntimeClosureTestRow {
    fn new(
        lane: WorthQueryLowerRuntimeClosureTestLane,
        digest: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            lane,
            digest: digest.into(),
            detail: detail.into(),
        }
    }

    pub fn lane(&self) -> WorthQueryLowerRuntimeClosureTestLane {
        self.lane
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeClosureTest {
    name: &'static str,
    certification_bundle: WorthQueryLowerRuntimeCertificationBundle,
    acceptance_suite: WorthQueryLowerRuntimeAcceptanceSuite,
    boundary_reconciliation: WorthQueryLowerRuntimeBoundaryReconciliationReport,
    proof_shape_audit: WorthQueryLowerRuntimeProofShapeAudit,
    rows: Vec<WorthQueryLowerRuntimeClosureTestRow>,
    suite_digest: String,
}

impl WorthQueryLowerRuntimeClosureTest {
    pub(crate) fn new(
        certification_bundle: WorthQueryLowerRuntimeCertificationBundle,
        acceptance_suite: WorthQueryLowerRuntimeAcceptanceSuite,
        boundary_reconciliation: WorthQueryLowerRuntimeBoundaryReconciliationReport,
        proof_shape_audit: WorthQueryLowerRuntimeProofShapeAudit,
        rows: Vec<WorthQueryLowerRuntimeClosureTestRow>,
    ) -> Self {
        let suite_digest = hash_parts(&[
            LOWER_RUNTIME_CLOSURE_TEST_NAME.to_string(),
            certification_bundle
                .certification_bundle_digest()
                .to_string(),
            acceptance_suite.suite_digest().to_string(),
            boundary_reconciliation.report_digest().to_string(),
            proof_shape_audit.proof_shape_digest().to_string(),
            hash_parts(
                &rows
                    .iter()
                    .map(|row| format!("{}|{}|{}", row.lane().as_str(), row.digest(), row.detail()))
                    .collect::<Vec<_>>(),
            ),
        ]);
        Self {
            name: LOWER_RUNTIME_CLOSURE_TEST_NAME,
            certification_bundle,
            acceptance_suite,
            boundary_reconciliation,
            proof_shape_audit,
            rows,
            suite_digest,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn certification_bundle(&self) -> &WorthQueryLowerRuntimeCertificationBundle {
        &self.certification_bundle
    }

    pub fn acceptance_suite(&self) -> &WorthQueryLowerRuntimeAcceptanceSuite {
        &self.acceptance_suite
    }

    pub fn boundary_reconciliation(&self) -> &WorthQueryLowerRuntimeBoundaryReconciliationReport {
        &self.boundary_reconciliation
    }

    pub fn proof_shape_audit(&self) -> &WorthQueryLowerRuntimeProofShapeAudit {
        &self.proof_shape_audit
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimeClosureTestRow] {
        &self.rows
    }

    pub fn suite_digest(&self) -> &str {
        &self.suite_digest
    }

    pub fn lane(
        &self,
        lane: WorthQueryLowerRuntimeClosureTestLane,
    ) -> &WorthQueryLowerRuntimeClosureTestRow {
        self.rows
            .iter()
            .find(|row| row.lane() == lane)
            .unwrap_or_else(|| panic!("missing closure test lane {}", lane.as_str()))
    }
}

pub fn worth_query_lower_runtime_closure_test(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeClosureTest {
    let certification_bundle = certify_lower_runtime_routing(resource_request);
    let acceptance_suite = worth_query_lower_runtime_acceptance_suite(resource_request);
    let boundary_reconciliation = worth_query_lower_runtime_boundary_reconciliation_report();
    let proof_shape_audit = worth_query_lower_runtime_proof_shape_audit();

    let control = acceptance_suite
        .lane(WorthQueryLowerRuntimeAcceptanceLane::Control)
        .clone();
    let hostile = acceptance_suite
        .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
        .clone();
    let parity = acceptance_suite
        .lane(WorthQueryLowerRuntimeAcceptanceLane::Parity)
        .clone();
    let former_specialist = certification_bundle
        .rows()
        .iter()
        .find(|row| {
            row.lane() == WorthQueryLowerRuntimeCertificationLane::FormerSpecialistSeamClosure
        })
        .expect("named closure test requires former specialist seam closure lane")
        .clone();
    let deferred_neighbor = certification_bundle
        .rows()
        .iter()
        .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::DeferredNeighborDenial)
        .expect("named closure test requires deferred neighbor denial lane")
        .clone();
    let downstream_boundary = certification_bundle
        .rows()
        .iter()
        .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::DownstreamBoundaryAudit)
        .expect("named closure test requires downstream boundary audit lane")
        .clone();

    assert_eq!(
        certification_bundle
            .rows()
            .iter()
            .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence)
            .map(|row| row.artifact_digest()),
        Some(acceptance_suite.suite_digest()),
        "named closure test requires bundle parity with acceptance evidence"
    );
    assert_eq!(
        certification_bundle
            .rows()
            .iter()
            .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence)
            .and_then(|row| row.failure_digest()),
        Some(hostile.digest()),
        "named closure test requires hostile acceptance parity"
    );
    assert_eq!(
        certification_bundle.output_digest("route_boundary_reconciliation_digest"),
        Some(boundary_reconciliation.report_digest()),
        "named closure test requires bundle parity with boundary reconciliation"
    );
    assert_eq!(
        certification_bundle.output_digest("route_proof_shape_digest"),
        Some(proof_shape_audit.proof_shape_digest()),
        "named closure test requires bundle parity with proof-shape audit"
    );

    WorthQueryLowerRuntimeClosureTest::new(
        certification_bundle,
        acceptance_suite,
        boundary_reconciliation,
        proof_shape_audit,
        vec![
            WorthQueryLowerRuntimeClosureTestRow::new(
                WorthQueryLowerRuntimeClosureTestLane::Control,
                control.digest(),
                control.detail(),
            ),
            WorthQueryLowerRuntimeClosureTestRow::new(
                WorthQueryLowerRuntimeClosureTestLane::Hostile,
                hash_parts(&[
                    hostile.digest().to_string(),
                    former_specialist.artifact_digest().to_string(),
                    deferred_neighbor
                        .failure_digest()
                        .expect("deferred neighbor lane should expose hostile digest")
                        .to_string(),
                ]),
                "hostile closure certifies deleted seam survival, former specialist seam survival, and deferred-neighbor widening remain forbidden".to_string(),
            ),
            WorthQueryLowerRuntimeClosureTestRow::new(
                WorthQueryLowerRuntimeClosureTestLane::Parity,
                parity.digest(),
                parity.detail(),
            ),
            WorthQueryLowerRuntimeClosureTestRow::new(
                WorthQueryLowerRuntimeClosureTestLane::DownstreamBoundary,
                downstream_boundary.artifact_digest(),
                downstream_boundary.detail(),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_test_has_named_control_hostile_and_parity_lanes() {
        let host_request =
            worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
                .serial_request(worth_execution::CancellationToken::new(), None);
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_closure_test(resource_request);

        assert_eq!(suite.name(), LOWER_RUNTIME_CLOSURE_TEST_NAME);
        assert_eq!(suite.rows().len(), 4);
        assert!(!suite.suite_digest().is_empty());
        assert_eq!(
            suite
                .lane(WorthQueryLowerRuntimeClosureTestLane::Control)
                .digest(),
            suite
                .acceptance_suite()
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Control)
                .digest()
        );
        assert_eq!(
            suite.boundary_reconciliation().report_digest(),
            suite
                .certification_bundle()
                .output_digest("route_boundary_reconciliation_digest")
                .expect("boundary reconciliation output should exist")
        );
        assert_eq!(
            suite.proof_shape_audit().proof_shape_digest(),
            suite
                .certification_bundle()
                .output_digest("route_proof_shape_digest")
                .expect("proof-shape output should exist")
        );
    }

    #[test]
    fn closure_test_binds_downstream_boundary_lane_to_certified_row() {
        let host_request =
            worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
                .serial_request(worth_execution::CancellationToken::new(), None);
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_closure_test(resource_request);

        assert_eq!(
            suite
                .lane(WorthQueryLowerRuntimeClosureTestLane::DownstreamBoundary)
                .digest(),
            suite
                .certification_bundle()
                .rows()
                .iter()
                .find(|row| row.lane()
                    == WorthQueryLowerRuntimeCertificationLane::DownstreamBoundaryAudit)
                .expect("downstream boundary row")
                .artifact_digest()
        );
    }

    #[test]
    fn closure_test_hostile_lane_aggregates_phase_seven_hostile_obligations() {
        let host_request =
            worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
                .serial_request(worth_execution::CancellationToken::new(), None);
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_closure_test(resource_request);
        let former_specialist = suite
            .certification_bundle()
            .rows()
            .iter()
            .find(|row| {
                row.lane() == WorthQueryLowerRuntimeCertificationLane::FormerSpecialistSeamClosure
            })
            .expect("former specialist seam closure row");
        let deferred_neighbor = suite
            .certification_bundle()
            .rows()
            .iter()
            .find(|row| {
                row.lane() == WorthQueryLowerRuntimeCertificationLane::DeferredNeighborDenial
            })
            .expect("deferred neighbor denial row");
        let expected = hash_parts(&[
            suite
                .acceptance_suite()
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
                .digest()
                .to_string(),
            former_specialist.artifact_digest().to_string(),
            deferred_neighbor
                .failure_digest()
                .expect("deferred neighbor lane should expose hostile digest")
                .to_string(),
        ]);

        assert_eq!(
            suite
                .lane(WorthQueryLowerRuntimeClosureTestLane::Hostile)
                .digest(),
            expected
        );
        assert_eq!(
            suite
                .lane(WorthQueryLowerRuntimeClosureTestLane::Parity)
                .digest(),
            suite
                .acceptance_suite()
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Parity)
                .digest()
        );
    }
}
