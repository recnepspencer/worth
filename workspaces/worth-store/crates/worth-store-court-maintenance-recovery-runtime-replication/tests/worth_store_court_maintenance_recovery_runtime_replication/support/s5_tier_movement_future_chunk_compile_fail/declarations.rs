pub(crate) use std::path::Path;

pub(crate) use {
    worth_store_test_support::compiler_boundary::cargo_dependency_manifest,
    worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite,
};

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[(
    "future_chunk_variant_cannot_be_constructed_directly.rs",
    &["cannot create non-exhaustive variant using struct expression"],
)];

pub(crate) fn run(fixtures: &[(&str, &[&str])]) {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "tier-movement-future-chunk",
        cargo_dependency_manifest(
            &[
                (
                    "worth-store-physical-isolation",
                    root.join("crates/worth-store-physical-isolation").as_path(),
                    &[],
                ),
                (
                    "worth-store-physical-format",
                    root.join("crates/worth-store-physical-format").as_path(),
                    &[],
                ),
            ],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join(
            "crates/worth-store-certification/tests/compile_fail/blobs/tier_movement_future_chunk",
        ),
        fixtures,
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), fixtures.len());
}

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-store-certification"
    ))
    .ancestors()
    .nth(2)
    .unwrap()
}
