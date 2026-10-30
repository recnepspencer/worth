pub(crate) use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) fn run_fixture(name: &str, expected: &[&str]) {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "physical-driver-contract-boundary", UiCompilerEnvironment::Certification, &root.join(
            "crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/compile_fail/recovery/driver_contract_boundary",
        ), &[(name, expected)])
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}
