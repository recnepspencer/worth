use super::surface::worth_query_lower_runtime_synthetic_tail_report;
use super::surface::{
    allowed_phase_six_synthetic_seams, required_phase_six_concrete_seams,
    worth_query_lower_runtime_acceptance_suite, worth_query_lower_runtime_representative_surface,
    WorthQueryLowerRuntimeAcceptanceLane,
};
use super::{
    certify_lower_runtime_performance_slopes, certify_lower_runtime_routing,
    worth_query_lower_runtime_boundary_reconciliation_report,
    worth_query_lower_runtime_certification_output_manifest,
    worth_query_lower_runtime_closeout_extension_outputs,
    worth_query_lower_runtime_closeout_report, worth_query_lower_runtime_closure_test,
    worth_query_lower_runtime_phase_artifact_manifest_digest,
    worth_query_lower_runtime_phase_manifest, worth_query_lower_runtime_phase_progression_digest,
    worth_query_lower_runtime_proof_shape_audit, worth_query_lower_runtime_proof_shape_digest,
    worth_query_lower_runtime_required_certification_outputs,
    worth_query_lower_runtime_typestate_transition_digest, WorthQueryLowerRuntimeCertificationLane,
    WorthQueryLowerRuntimePhaseArtifact,
};
use crate::identity::hash_parts;
use crate::lower_runtime_routing::{
    certify_lower_runtime_non_bypass, worth_query_lower_runtime_crossing_inventory,
    worth_query_lower_runtime_gap_registry, worth_query_lower_runtime_support_matrix,
};

#[test]
fn certification_bundle_contains_phase_seven_lanes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);

    for lane in [
        WorthQueryLowerRuntimeCertificationLane::CrossingsSurface,
        WorthQueryLowerRuntimeCertificationLane::BoundaryClosureSurface,
        WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence,
        WorthQueryLowerRuntimeCertificationLane::SyntheticTailPolicy,
        WorthQueryLowerRuntimeCertificationLane::RouteParity,
        WorthQueryLowerRuntimeCertificationLane::FormerSpecialistSeamClosure,
        WorthQueryLowerRuntimeCertificationLane::DeferredNeighborDenial,
        WorthQueryLowerRuntimeCertificationLane::DownstreamBoundaryAudit,
        WorthQueryLowerRuntimeCertificationLane::ProofShapeSurface,
        WorthQueryLowerRuntimeCertificationLane::Performance,
    ] {
        assert!(bundle.rows().iter().any(|row| row.lane() == lane));
    }
    assert!(!bundle.certification_bundle_digest().is_empty());
}

#[path = "tests/certification_bundle_emits_required_outputs.rs"]
mod certification_bundle_emits_required_outputs;

#[test]
fn certification_output_manifest_extends_required_outputs_exactly() {
    assert_eq!(
        worth_query_lower_runtime_required_certification_outputs().len()
            + worth_query_lower_runtime_closeout_extension_outputs().len(),
        worth_query_lower_runtime_certification_output_manifest().len()
    );
}

#[test]
fn phase_manifest_is_public_and_consumable_by_closeout_bundle() {
    let manifest = worth_query_lower_runtime_phase_manifest();

    assert_eq!(
        manifest.manifest_digest(),
        worth_query_lower_runtime_phase_artifact_manifest_digest()
    );
    assert_eq!(
        manifest.typestate_transition_digest(),
        worth_query_lower_runtime_typestate_transition_digest()
    );
    assert_eq!(
        manifest
            .rows()
            .last()
            .expect("manifest rows")
            .next_consumer(),
        "runtime-api-public-stabilization gate"
    );
}

#[test]
fn stabilization_closeout_report_is_public_and_consumes_final_phase_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let report = worth_query_lower_runtime_closeout_report(request_execution);

    assert_eq!(
        report
            .phase_manifest()
            .rows()
            .get(15)
            .expect("named closure test row")
            .artifact(),
        WorthQueryLowerRuntimePhaseArtifact::NamedClosureTest
    );
    assert_eq!(
        report
            .phase_manifest()
            .rows()
            .last()
            .expect("manifest rows")
            .artifact(),
        WorthQueryLowerRuntimePhaseArtifact::StabilizationCloseoutReport
    );
    assert_eq!(
        report
            .phase_manifest()
            .rows()
            .last()
            .expect("manifest rows")
            .next_consumer(),
        "runtime-api-public-stabilization gate"
    );
    assert_eq!(
        report.closure_test().suite_digest(),
        worth_query_lower_runtime_closure_test(request_execution).suite_digest()
    );
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
            .output_digest("route_boundary_reconciliation_digest"),
        Some(report.boundary_reconciliation().report_digest())
    );
    assert_eq!(
        report
            .certification_bundle()
            .output_digest("route_synthetic_tail_report_digest"),
        Some(report.synthetic_tail_report().report_digest())
    );
}

#[test]
fn certification_bundle_acceptance_lane_matches_named_suite() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);
    let suite = worth_query_lower_runtime_acceptance_suite(request_execution);
    let row = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence)
        .expect("acceptance evidence lane should exist");

    assert_eq!(row.artifact_digest(), suite.suite_digest());
    assert_eq!(
        row.failure_digest(),
        Some(
            suite
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
                .digest()
        )
    );
}

#[test]
fn certification_bundle_failure_digest_is_hostile_row_aggregate() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);
    let expected = hash_parts(
        &bundle
            .rows()
            .iter()
            .filter_map(|row| row.failure_digest().map(str::to_string))
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        bundle.output_digest("failure_digest"),
        Some(expected.as_str())
    );
}

#[test]
fn parity_digest_changes_when_intentionally_different_route_families_are_compared() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);
    let route_parity = bundle
        .output_digest("route_parity_digest")
        .expect("route parity output should exist");
    let route_failure = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == WorthQueryLowerRuntimeCertificationLane::DownstreamBoundaryAudit)
        .and_then(|row| row.failure_digest())
        .expect("downstream boundary hostile digest should exist");

    assert_ne!(
        route_parity, route_failure,
        "parity control digest must not collapse into hostile divergence evidence"
    );
}

#[test]
fn proof_shape_and_slope_surfaces_stay_exported() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let proof = worth_query_lower_runtime_proof_shape_audit();
    let surface = worth_query_lower_runtime_representative_surface(request_execution);
    let slopes = certify_lower_runtime_performance_slopes(&surface);

    assert_eq!(proof.rows().len(), 5);
    assert_eq!(slopes.rows().len(), 6);
    assert!(slopes
        .digest_for_output("debt_registry_lookup_slope_digest")
        .is_some());
    assert_eq!(slopes.profiles().len(), 3);
}

#[test]
fn compatibility_debt_registry_digest_tracks_closed_gap_registry() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);
    let gaps = worth_query_lower_runtime_gap_registry();

    assert_eq!(gaps.rows().len(), 0);
    assert_eq!(
        bundle.output_digest("compatibility_debt_registry_digest"),
        Some(gaps.registry_digest().as_str())
    );
}

#[test]
fn certification_bundle_phase_six_required_seams_are_concrete() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let surface =
        super::surface::worth_query_lower_runtime_representative_surface(request_execution);

    for seam_key in required_phase_six_concrete_seams() {
        assert_eq!(
            surface.evidence_source_for(*seam_key),
            Some(
                super::surface::WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture
            ),
            "required phase six seam {} must remain runtime-backed",
            seam_key.as_str()
        );
    }
}
