pub(crate) use {std::path::Path};

pub(crate) use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store-certification"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}