use super::evidence::worth_query_lower_runtime_representative_surface;

#[test]
fn representative_surface_reports_concrete_and_synthetic_coverage_widths() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);

    assert_eq!(
        surface.concrete_surface_width() + surface.synthetic_surface_width(),
        surface.envelopes().len()
    );
    assert!(surface.concrete_surface_width() >= 22);
    assert!(!surface.concrete_surface_digest().is_empty());
    assert_eq!(surface.synthetic_surface_width(), 0);
    assert!(surface.synthetic_surface_seams().is_empty());
    assert_eq!(
        surface.synthetic_surface_digest(),
        crate::identity::hash_parts(&Vec::<String>::new())
    );
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"public-live-view-declaration"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"subscription-continuity"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"basis-readmission-from-truth-view-evidence"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"basis-readmission-from-subscription-evidence"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"historical-bridge-lowering"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"effect-backed-relational-mutation"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"effect-backed-relational-merge"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"effect-backed-bridge-writeback"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"runtime-intent-authority-adapter"));
    assert!(!surface
        .synthetic_surface_seams()
        .contains(&"intent-runtime-execution"));
}
