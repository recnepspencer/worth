#[allow(unused_imports)] use crate::s6_secure_io_authority_compile_fail::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn io_qos_secure_io_authority_rejects_lower_authority_sources() {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "secure-io-authority",
        cargo_dependency_manifest(
            &[
                (
                    "worth-store-aspect-native",
                    root.join("crates/worth-store-aspect-native").as_path(),
                    &[],
                ),
                (
                    "worth-store-contracts",
                    root.join("crates/worth-store-contracts").as_path(),
                    &[],
                ),
                (
                    "worth-store-io-scheduler",
                    root.join("crates/worth-store-io-scheduler").as_path(),
                    &[],
                ),
                (
                    "worth-store-security",
                    root.join("crates/worth-store-security").as_path(),
                    &[],
                ),
            ],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join(
            "crates/worth-store-certification/tests/compile_fail/scheduling/secure_io_authority",
        ),
        FIXTURES,
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
