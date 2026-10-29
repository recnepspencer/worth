pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

pub(crate) fn assert_compile_fails(fixture_name: &str, expected_stderr: &[&str], extern_crates: &[&str]) {
    assert_compile_fails_in_ui_dir("foundations", fixture_name, expected_stderr, extern_crates);
}

pub(crate) fn assert_compile_fails_in_ui_dir(
    ui_dir: &str,
    fixture_name: &str,
    expected_stderr: &[&str],
    extern_crates: &[&str],
) {
    let root = store_workspace_root();
    let dependencies = dependency_declarations(root, extern_crates);
    let borrowed = dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.name.as_str(),
                dependency.path.as_path(),
                dependency.features.as_slice(),
            )
        })
        .collect::<Vec<_>>();
    let evidence = run_cargo_ui_fixture_suite(
        root,
        &format!("layout-indexes-{ui_dir}"),
        cargo_dependency_manifest(&borrowed, &[]),
        "layout-certification-authority",
        "diagnostic-test",
        &root
            .join("crates/worth-store-layout-indexes/tests/compile_fail/layout")
            .join(ui_dir),
        &[(fixture_name, expected_stderr)],
    )
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}

pub(crate) struct DependencyDeclaration {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) features: Vec<&'static str>,
}

pub(crate) fn dependency_declarations(root: &Path, extern_crates: &[&str]) -> Vec<DependencyDeclaration> {
    let mut names = std::collections::BTreeSet::from(["worth_store_layout_indexes"]);
    names.extend(extern_crates.iter().copied());
    names
        .into_iter()
        .map(|name| dependency(root, name))
        .collect()
}

pub(crate) fn dependency(root: &Path, rust_name: &str) -> DependencyDeclaration {
    let package = rust_name.replace('_', "-");
    let forge_root = root.ancestors().nth(2).unwrap();
    let path = if matches!(rust_name, "worth_foundational" | "worth_proof") {
        forge_root.join("crates").join(&package)
    } else {
        root.join("crates").join(&package)
    };
    let features = match rust_name {
        "worth_store_blob_chunks" | "worth_store_io_scheduler" => {
            vec!["certification-test-authority"]
        }
        "worth_store_security" => vec!["certification-test-authority"],
        "worth_store_test_support" => vec!["boundary-fixtures"],
        "worth_store_wal" => vec!["certification-authority"],
        _ => Vec::new(),
    };
    DependencyDeclaration {
        name: package,
        path,
        features,
    }
}

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-store-layout-indexes"))
        .ancestors()
        .nth(2)
        .expect("layout-indexes crate lives under the Store workspace")
}