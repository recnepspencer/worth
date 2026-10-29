#[allow(unused_imports)] use crate::aspect_native_authority_ui::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn aspect_native_authority_denies_raw_public_callers() {
    let root = store_workspace_root();
    let forge_root = root.ancestors().nth(2).unwrap();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "aspect-native-authority",
        cargo_dependency_manifest(
            &[
                (
                    "worth-foundational",
                    forge_root.join("crates/worth-foundational").as_path(),
                    &[],
                ),
                (
                    "worth-store-aspect-native",
                    root.join("crates/worth-store-aspect-native").as_path(),
                    &[],
                ),
            ],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join("crates/worth-store-certification/tests/ui/aspect_native_authority"),
        FIXTURES,
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
