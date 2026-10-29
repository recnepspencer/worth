pub(crate) use {std::path::Path};

pub(crate) use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

pub(crate) fn run_case(suite: &str, blob_authority: bool, fixture: &str, expected: &[&str]) {
    let root = store_workspace_root();
    let dependencies = [
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

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store-physical-certification"))
        .ancestors()
        .nth(2)
        .unwrap()
}