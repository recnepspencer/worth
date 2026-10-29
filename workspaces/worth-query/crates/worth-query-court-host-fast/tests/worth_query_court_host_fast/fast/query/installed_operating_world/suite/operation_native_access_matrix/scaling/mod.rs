pub(crate) use worth_query_worlds::facade::query::suite::operation_native_access_matrix::scaling::*;

#[test]
fn native_access_work_is_invariant_to_unrelated_rows_facts_views_and_domains() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let baseline = run_case(
        request_execution,
        "installed-native-scale-baseline",
        1,
        false,
        false,
    );
    let world_scaled = run_case(
        request_execution,
        "installed-native-scale-world-pressure",
        64,
        false,
        true,
    );
    let scaled = run_case(
        request_execution,
        "installed-native-scale-pressure",
        64,
        true,
        true,
    );

    assert_eq!(baseline.journey, world_scaled.journey);
    assert_eq!(baseline.access, scaled.access);
    assert_eq!(baseline.resolution, scaled.resolution);
    assert_eq!(baseline.binding.fact_scans, 0);
    assert_eq!(scaled.binding.fact_scans, 0);
    assert_eq!(baseline.binding.row_scans, 0);
    assert_eq!(scaled.binding.row_scans, 0);
    assert_eq!(baseline.binding.path_parses, 0);
    assert_eq!(scaled.binding.path_parses, 0);
    assert_eq!(baseline.binding.view_registry_inspections, 0);
    assert_eq!(scaled.binding.view_registry_inspections, 0);
    assert_eq!(baseline.binding.domain_registry_inspections, 0);
    assert_eq!(scaled.binding.domain_registry_inspections, 0);
    assert_eq!(baseline.access.view_registry_inspections, 0);
    assert_eq!(scaled.access.view_registry_inspections, 0);
    assert_eq!(baseline.access.domain_registry_inspections, 0);
    assert_eq!(scaled.access.domain_registry_inspections, 0);
    assert_eq!(baseline.resolution.key_scans, 0);
    assert_eq!(scaled.resolution.key_scans, 0);
    assert_eq!(baseline.resolution.path_matches, 0);
    assert_eq!(scaled.resolution.path_matches, 0);
    assert_eq!(baseline.resolution.path_parses, 0);
    assert_eq!(scaled.resolution.path_parses, 0);
    assert_eq!(baseline.indexed_accesses, baseline.declared_width);
    assert_eq!(scaled.indexed_accesses, scaled.declared_width);
    assert_eq!(baseline.refinement_checks, baseline.declared_width);
    assert_eq!(scaled.refinement_checks, scaled.declared_width);
    assert_eq!(baseline.resolution_lookups, baseline.declared_width * 2);
    assert_eq!(scaled.resolution_lookups, scaled.declared_width * 2);
    assert_eq!(baseline.binding.declared_key_routes, 1);
    assert_eq!(scaled.binding.declared_key_routes, scalar_samples().len());
    assert_eq!(baseline.binding.declared_key_layout_checks, 1);
    assert_eq!(
        scaled.binding.declared_key_layout_checks,
        scalar_samples().len()
    );
    assert_eq!(baseline.binding.lane_shape_checks, 2);
    assert_eq!(scaled.binding.lane_shape_checks, 2);
}

#[test]
fn native_key_access_is_constant_while_declared_width_is_charged() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    // The expanded case adds unrelated world content and declares more keys
    // and rows. Each key still has its exact access and declaration prices.
    let baseline = run_case(request_execution, "native-one-key", 1, false, false);
    let expanded = run_case(request_execution, "native-all-keys", 64, true, true);
    assert_eq!(baseline.declared_width, 1);
    assert_eq!(expanded.declared_width, scalar_samples().len());
    assert_eq!(baseline.access, expanded.access);
    assert_eq!(baseline.resolution, expanded.resolution);
    for case in [&baseline, &expanded] {
        let one_route_per_key = case.declared_width;
        let one_layout_check_per_key = case.declared_width;
        assert_eq!(case.binding.declared_key_routes, one_route_per_key);
        assert_eq!(
            case.binding.declared_key_layout_checks,
            one_layout_check_per_key
        );
        assert_eq!(
            case.journey["query.native_binding.declared_key_routes"].0,
            one_route_per_key as u64
        );
        assert_eq!(
            case.journey["query.native_binding.declared_key_layout_checks"].0,
            one_layout_check_per_key as u64
        );
        assert_eq!(case.binding.fact_scans, 0);
        assert_eq!(case.binding.row_scans, 0);
        assert_eq!(case.binding.path_parses, 0);
        assert_eq!(case.binding.view_registry_inspections, 0);
        assert_eq!(case.binding.domain_registry_inspections, 0);
        assert_eq!(case.access.view_registry_inspections, 0);
        assert_eq!(case.access.domain_registry_inspections, 0);
        assert_eq!(case.resolution.key_scans, 0);
        assert_eq!(case.resolution.path_matches, 0);
        assert_eq!(case.resolution.path_parses, 0);
        assert_eq!(case.indexed_accesses, case.declared_width);
        assert_eq!(case.refinement_checks, case.declared_width);
        let two_resolution_lookups_per_key = case.declared_width * 2;
        assert_eq!(case.resolution_lookups, two_resolution_lookups_per_key);
        assert_eq!(case.binding.lane_shape_checks, 2);
    }
}
