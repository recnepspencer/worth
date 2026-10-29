#[allow(unused_imports)] use crate::backend_qualification_public_boundary_compile_fail::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn copied_rows_and_proof_authority_cannot_enter_public_publication_boundary() {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "backend-qualification-public-boundary",
        cargo_dependency_manifest(
            &[
                ("worth-store-physical-backend", root.join("crates/worth-store-physical-backend").as_path(), &["certification-test-authority"]),
                ("worth-store-physical-certification", root.join("crates/worth-store-physical-certification").as_path(), &[]),
            ],
            &[],
        ),
        "backend-certification-authority",
        "diagnostic-test",
        &root.join("crates/worth-store-physical-certification/tests/ui/qualification/backend_public_boundary"),
        FIXTURES,
    ).unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
