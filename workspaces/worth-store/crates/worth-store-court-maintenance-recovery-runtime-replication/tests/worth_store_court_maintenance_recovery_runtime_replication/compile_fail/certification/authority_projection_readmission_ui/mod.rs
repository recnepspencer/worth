#[allow(unused_imports)] use crate::authority_projection_readmission_ui::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn authority_projection_readmission_denies_lower_authority_inputs() {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "authority-projection-readmission",
        cargo_dependency_manifest(
            &[
                (
                    "worth-store-aspect-native",
                    root.join("crates/worth-store-aspect-native").as_path(),
                    &[],
                ),
                (
                    "worth-store-authority",
                    root.join("crates/worth-store-authority").as_path(),
                    &[],
                ),
                (
                    "worth-store-contracts",
                    root.join("crates/worth-store-contracts").as_path(),
                    &[],
                ),
            ],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join("crates/worth-store-certification/tests/ui/authority_projection_readmission"),
        FIXTURES,
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
