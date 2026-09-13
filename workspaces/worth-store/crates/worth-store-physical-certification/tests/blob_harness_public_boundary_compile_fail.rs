use std::path::Path;

use worth_store_test_support::compiler_boundary::{
    cargo_dependency_manifest, run_cargo_ui_fixture_suite,
};

#[test]
fn blob_harness_envelope_raw_constructor_is_not_public() {
    run_case(
        "blob-harness-envelope",
        false,
        "blob_harness_envelope_raw_constructor_is_private.rs",
        &["new", "private"],
    );
}

#[test]
fn blob_harness_executed_witness_cannot_be_forged_from_raw_fields() {
    run_case(
        "blob-harness-witness",
        true,
        "blob_harness_executed_witness_fields_are_private.rs",
        &["BlobHarnessExecutedWitness", "private"],
    );
}

#[test]
fn blob_harness_execution_authority_is_not_public_on_default_surface() {
    run_case(
        "blob-harness-default",
        false,
        "blob_harness_execution_authority_is_not_public.rs",
        &["execute_blob_harness", "BlobHarnessExecutionInput"],
    );
}

fn run_case(suite: &str, blob_authority: bool, fixture: &str, expected: &[&str]) {
    let root = store_workspace_root();
    let dependencies = vec![
        (
            "worth-store-blob-chunks",
            root.join("crates/worth-store-blob-chunks"),
            blob_authority
                .then_some(vec!["certification-test-authority"])
                .unwrap_or_default(),
        ),
        (
            "worth-store-budgets",
            root.join("crates/worth-store-budgets"),
            Vec::new(),
        ),
    ];
    let borrowed = dependencies
        .iter()
        .map(|(name, path, features)| (*name, path.as_path(), features.as_slice()))
        .collect::<Vec<_>>();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        suite,
        cargo_dependency_manifest(&borrowed, &[]),
        if blob_authority {
            "blob-certification-authority"
        } else {
            "production"
        },
        "diagnostic-test",
        &root.join(
            "crates/worth-store-physical-certification/tests/ui/blob_harness/public_boundary",
        ),
        &[(fixture, expected)],
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

fn store_workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}
