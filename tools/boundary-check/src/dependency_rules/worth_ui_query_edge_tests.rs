use super::*;

fn contract() -> crate::config::WorthUiQueryEdgeContract {
    crate::config::WorthUiQueryEdgeContract {
        workspace: "worth-ui".to_owned(),
        engine_package: "worth-query".to_owned(),
        allowed_production_consumers: vec!["worth-ui-query-binding".to_owned()],
        guidance: "consume binding-owned artifacts".to_owned(),
    }
}

fn fixture_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "worth-ui-query-edge-{label}-{}",
        std::process::id()
    ))
}

fn write_crate(root: &Path, package: &str, has_dependency: bool, source: &str) {
    let crate_root = root.join("worth-ui").join("crates").join(package);
    std::fs::create_dir_all(crate_root.join("src")).expect("fixture source directory");
    let dependency = if has_dependency {
        "worth-query = { path = \"../../worth-query\" }"
    } else {
        ""
    };
    std::fs::write(
        crate_root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\n[dependencies]\n{dependency}\n"
        ),
    )
    .expect("fixture manifest");
    std::fs::write(crate_root.join("src/lib.rs"), source).expect("fixture source");
}

#[test]
fn binding_crate_is_the_only_admitted_production_query_edge() {
    let root = fixture_root("allowed");
    write_crate(
        &root,
        "worth-ui-query-binding",
        true,
        "pub fn binding_edge() {}",
    );
    assert!(validate_worth_ui_query_edge(&root, &contract())
        .expect("edge validation")
        .is_empty());
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn direct_runtime_or_host_dependency_reports_the_admitted_path() {
    for package in ["worth-ui-runtime", "worth-ui-host-contract"] {
        let root = fixture_root(package);
        write_crate(&root, package, true, "pub fn forbidden_query_edge() {}");
        let diagnostics =
            validate_worth_ui_query_edge(&root, &contract()).expect("edge validation");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .message()
            .contains("consume binding-owned artifacts"));
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }
}

#[test]
fn raw_facade_reexport_reports_the_admitted_path() {
    let root = fixture_root("facade-reexport");
    write_crate(
        &root,
        "worth-ui",
        false,
        "pub use worth_query::facade::read::*;",
    );
    let diagnostics = validate_worth_ui_query_edge(&root, &contract()).expect("edge validation");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message().contains("raw `worth_query`"));
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn renamed_engine_dependency_is_still_denied() {
    let root = fixture_root("renamed-engine");
    write_crate(
        &root,
        "worth-ui-runtime",
        false,
        "pub fn hidden_alias_edge() {}",
    );
    let manifest = root
        .join("worth-ui")
        .join("crates")
        .join("worth-ui-runtime")
        .join("Cargo.toml");
    std::fs::write(
        manifest,
        "[package]\nname = \"worth-ui-runtime\"\nversion = \"0.1.0\"\n[target.'cfg(windows)'.dependencies]\nquery_engine = { package = \"worth-query\", path = \"../../worth-query\" }\n",
    )
    .expect("aliased fixture manifest");
    let diagnostics = validate_worth_ui_query_edge(&root, &contract()).expect("edge validation");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0]
        .message()
        .contains("direct production dependency on `worth-query`"));
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}
