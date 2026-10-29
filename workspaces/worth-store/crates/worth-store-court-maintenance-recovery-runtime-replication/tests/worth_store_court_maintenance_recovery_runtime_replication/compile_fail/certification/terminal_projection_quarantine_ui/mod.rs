#[allow(unused_imports)] use crate::terminal_projection_quarantine_ui::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn terminal_projection_quarantine_denies_neutral_public_callers() {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "terminal-projection-quarantine",
        cargo_dependency_manifest(
            &[(
                "worth-store-aspect-native",
                root.join("crates/worth-store-aspect-native").as_path(),
                &[],
            )],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join("crates/worth-store-certification/tests/ui/terminal_projection_quarantine"),
        FIXTURES,
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
