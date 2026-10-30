use std::path::Path;

use worth_store_test_support::compiler_boundary::{read_ui_fixture_suite, UiCompilerEnvironment};

pub(crate) fn assert_compile_fails(
    environment: UiCompilerEnvironment,
    fixture_name: &str,
    expected_stderr: &[&str],
) {
    assert_compile_fails_in_ui_dir(environment, "foundations", fixture_name, expected_stderr);
}

pub(crate) fn assert_compile_fails_in_ui_dir(
    environment: UiCompilerEnvironment,
    ui_dir: &str,
    fixture_name: &str,
    expected_stderr: &[&str],
) {
    let root = store_workspace_root();
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving/sources/worth_store_layout_indexes/compile_fail/layout")
        .join(ui_dir);
    let evidence = read_ui_fixture_suite(
        root,
        &format!("layout-indexes-{ui_dir}"),
        environment,
        &source_root,
        &[(fixture_name, expected_stderr)],
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

pub(crate) fn store_workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("layout compiler court lives under the Store workspace")
}
