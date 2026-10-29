pub(crate) use {std::path::Path};

pub(crate) use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

pub(crate) fn run_fixture(name: &str, expected: &[&str]) {
    let root = store_workspace_root();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        "physical-driver-contract-boundary",
        cargo_dependency_manifest(
            &[(
                "worth-store-physical-certification",
                root.join("crates/worth-store-physical-certification")
                    .as_path(),
                &[],
            )],
            &[],
        ),
        "production",
        "diagnostic-test",
        &root.join(
            "crates/worth-store-certification/tests/compile_fail/recovery/driver_contract_boundary",
        ),
        &[(name, expected)],
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store-certification"))
        .ancestors()
        .nth(2)
        .expect("certification crate lives under the Store workspace")
}