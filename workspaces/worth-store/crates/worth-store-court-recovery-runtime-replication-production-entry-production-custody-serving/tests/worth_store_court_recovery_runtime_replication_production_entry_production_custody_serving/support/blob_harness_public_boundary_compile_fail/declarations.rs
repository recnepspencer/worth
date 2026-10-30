use std::path::Path;

use worth_store_test_support::compiler_boundary::{read_ui_fixture_suite, UiCompilerEnvironment};

pub(crate) fn run_case(
    suite: &str,
    environment: UiCompilerEnvironment,
    fixture: &str,
    expected: &[&str],
) {
    let root = store_workspace_root();
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving/sources/worth_store_physical_certification/ui/blob_harness/public_boundary");
    let evidence = read_ui_fixture_suite(
        root,
        suite,
        environment,
        &source_root,
        &[(fixture, expected)],
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

pub(crate) fn store_workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("blob compiler court lives under the Store workspace")
}
