pub(crate) use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[(
    "future_chunk_variant_cannot_be_constructed_directly.rs",
    &["cannot create non-exhaustive variant using struct expression"],
)];

pub(crate) fn run(fixtures: &[(&str, &[&str])]) {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "tier-movement-future-chunk", UiCompilerEnvironment::Production, &root.join(
            "crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/compile_fail/blobs/tier_movement_future_chunk",
        ), fixtures)
    .unwrap();
    assert_eq!(evidence.fixtures.len(), fixtures.len());
}

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}
