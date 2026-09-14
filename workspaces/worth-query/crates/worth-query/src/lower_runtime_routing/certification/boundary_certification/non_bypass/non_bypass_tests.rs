use super::*;

#[test]
fn certify_lower_runtime_non_bypass_passes_for_current_query_topology() {
    let audit = certify_lower_runtime_non_bypass()
        .expect("current workspace should satisfy the lower-runtime non-bypass audit");
    assert_eq!(
        audit.route_public_surface_digest(),
        worth_query_lower_runtime_public_surface_inventory().public_surface_digest()
    );
    assert!(audit.checked_file_count() >= 4);
}

#[test]
fn routed_surface_paths_cover_mutation_and_batch_boundary_files() {
    let paths = routed_surface_scan_targets();

    for path in [
        "crates/worth-query/src/runtime/read_composition_runtime.rs",
        "crates/worth-query/src/runtime/workspace.rs",
        "crates/worth-query/src/runtime/runtime_writes.rs",
        "crates/worth-query/src/runtime/runtime_batch_write_entrypoints.rs",
        "crates/worth-query/src/runtime/runtime_batch_writes.rs",
        "crates/worth-query/src/runtime/backend/*",
    ] {
        assert!(
            paths.iter().any(|(candidate, _)| *candidate == path),
            "missing routed surface path {path}"
        );
    }
}

#[test]
fn routed_surface_scan_targets_reconcile_remaining_phase_six_seam_files() {
    let targets = routed_surface_scan_targets();

    for (path, allow_imports) in [
        ("crates/worth-query/src/runtime/backend/*", true),
        (
            "crates/worth-query/src/runtime/read_composition_runtime.rs",
            false,
        ),
        (
            "crates/worth-query-admission/src/domain_computation/basis_lifecycle/lower_runtime/mod.rs",
            false,
        ),
        ("crates/worth-query/src/historical/bridge_lowering.rs", true),
        (
            "crates/worth-query/src/projection_consumption/source/mod.rs",
            true,
        ),
        (
            "crates/worth-query/src/runtime/inspection/causal/builder_bridge.rs",
            true,
        ),
        ("crates/worth-query/src/frontier_signal_adapter.rs", true),
        ("crates/worth-query/src/effect_lifecycle/execution.rs", true),
        (
            "crates/worth-query/src/effect_lifecycle/execution_bridge.rs",
            true,
        ),
        (
            "crates/worth-query/src/runtime/backend/intent_authority.rs",
            true,
        ),
    ] {
        assert!(
            targets
                .iter()
                .any(|(candidate, allow)| *candidate == path && *allow == allow_imports),
            "missing routed scan target {path} with allow_imports={allow_imports}"
        );
    }
}

#[test]
fn hostile_workspace_mutation_surface_outside_routed_lane_is_rejected() {
    assert_hostile_import_rejected(
        "target/lower_runtime_hostile_workspace_write.rs",
        "use worth_runtime_bridge::facade::RuntimeBridge;\n",
        "crates/worth-query/src/runtime/workspace.rs",
    );
}

#[test]
fn hostile_runtime_batch_surface_outside_routed_lane_is_rejected() {
    assert_hostile_import_rejected(
        "target/lower_runtime_hostile_runtime_batch.rs",
        "use worth_signal::facade::SignalInvalidationScope;\n",
        "crates/worth-query/src/runtime/runtime_batch_writes.rs",
    );
}

fn assert_hostile_import_rejected(temp_relative: &str, source: &str, scanned_relative: &str) {
    let workspace_root = workspace_root().expect("workspace root should resolve");
    let mut hostile = Vec::new();
    let temp = workspace_root.join(temp_relative);

    std::fs::create_dir_all(temp.parent().expect("hostile fixture must have a parent"))
        .expect("hostile fixture directory should exist");
    std::fs::write(&temp, source).expect("hostile fixture should write");
    scan_file_contents(&temp, scanned_relative, false, &mut hostile)
        .expect("hostile fixture should scan");
    std::fs::remove_file(&temp).expect("hostile fixture should clean up");

    assert_eq!(hostile.len(), 1);
    assert!(hostile[0].contains("outside the declared routed boundary"));
}
