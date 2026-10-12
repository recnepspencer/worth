//! certification bundle emits required outputs.
use super::*;

#[test]
fn certification_bundle_emits_required_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_lower_runtime_routing(request_execution);
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let support = worth_query_lower_runtime_support_matrix();
    let non_bypass = certify_lower_runtime_non_bypass().expect("non-bypass should pass");
    let reconciliation = worth_query_lower_runtime_boundary_reconciliation_report();
    let synthetic_tail = worth_query_lower_runtime_synthetic_tail_report(request_execution);
    let surface = worth_query_lower_runtime_representative_surface(request_execution);
    let slopes = certify_lower_runtime_performance_slopes(&surface);
    let reconciliation_width = reconciliation.rows().len().to_string();
    let synthetic_tail_width = synthetic_tail.rows().len().to_string();
    let full_crossing_width = slopes
        .full_profile()
        .counters()
        .crossing_inventory_width()
        .to_string();
    let full_route_width = slopes
        .full_profile()
        .counters()
        .route_plan_width()
        .to_string();
    let full_evidence_width = slopes
        .full_profile()
        .counters()
        .boundary_evidence_width()
        .to_string();

    for output in worth_query_lower_runtime_certification_output_manifest() {
        assert!(
            bundle.output_digest(output).is_some(),
            "missing required output {output}"
        );
    }
    assert_eq!(
        bundle.output_digests().len(),
        worth_query_lower_runtime_certification_output_manifest().len()
    );
    assert_eq!(
        bundle.output_digest("crossing_inventory_digest"),
        Some(crossings.inventory_digest().as_str())
    );
    assert_eq!(
        bundle.output_digest("route_support_matrix_digest"),
        Some(support.matrix_digest().as_str())
    );
    assert_eq!(
        bundle.output_digest("route_non_bypass_digest"),
        Some(non_bypass.route_non_bypass_digest())
    );
    assert_eq!(
        bundle.output_digest("route_boundary_reconciliation_digest"),
        Some(reconciliation.report_digest())
    );
    assert_eq!(
        bundle.output_digest("route_phase_artifact_manifest_digest"),
        Some(worth_query_lower_runtime_phase_artifact_manifest_digest().as_str())
    );
    assert_eq!(
        bundle.output_digest("route_proof_shape_digest"),
        Some(worth_query_lower_runtime_proof_shape_digest().as_str())
    );
    assert_eq!(
        bundle.output_digest("route_phase_progression_digest"),
        Some(worth_query_lower_runtime_phase_progression_digest().as_str())
    );
    assert_eq!(
        bundle.output_digest("route_typestate_transition_digest"),
        Some(worth_query_lower_runtime_typestate_transition_digest().as_str())
    );
    assert_ne!(
        bundle.output_digest("query_digest"),
        bundle.output_digest("capability_request_digest"),
        "query identity must not collapse into request identity"
    );
    let concrete_width = bundle
        .output_digest("route_concrete_surface_width")
        .expect("concrete width output should exist")
        .parse::<usize>()
        .expect("concrete width should parse as usize");
    let synthetic_width = bundle
        .output_digest("route_synthetic_surface_width")
        .expect("synthetic width output should exist")
        .parse::<usize>()
        .expect("synthetic width should parse as usize");
    assert_eq!(
        concrete_width + synthetic_width,
        crossings.rows().len(),
        "concrete and synthetic surface widths should partition crossing inventory"
    );
    assert!(
        concrete_width >= required_phase_six_concrete_seams().len(),
        "phase six certification should expose every required concrete seam"
    );
    assert_ne!(
        bundle.output_digest("route_concrete_surface_digest"),
        bundle.output_digest("route_synthetic_surface_digest"),
        "concrete and synthetic surfaces must remain distinguishable in the bundle"
    );
    assert_eq!(
        synthetic_width,
        allowed_phase_six_synthetic_seams().len(),
        "synthetic width must match the explicit phase six allowlist"
    );
    assert_eq!(
        bundle.output_digest("route_synthetic_tail_report_digest"),
        Some(synthetic_tail.report_digest())
    );
    assert_eq!(
        bundle.output_digest("route_synthetic_tail_justification_digest"),
        Some(synthetic_tail.justification_digest())
    );
    assert_eq!(
        bundle.output_digest("route_boundary_reconciliation_width"),
        Some(reconciliation_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("route_synthetic_tail_width"),
        Some(synthetic_tail_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("counter_snapshot"),
        Some(slopes.full_profile().counters().counter_snapshot_digest())
    );
    assert_eq!(
        bundle.output_digest("crossing_inventory_width"),
        Some(full_crossing_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("route_plan_width"),
        Some(full_route_width.as_str())
    );
    assert_eq!(
        bundle.output_digest("boundary_evidence_width"),
        Some(full_evidence_width.as_str())
    );
}
