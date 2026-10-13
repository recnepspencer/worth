fn debug_diagnostics(source: &str) -> Vec<crate::diagnostics::Diagnostic> {
    let root = worth_harness::tooling::fixture_path::unique_path("debug-inventory-crate");
    let crate_root = root.join("workspaces/worth-query/crates/worth-query-execution");
    std::fs::create_dir_all(crate_root.join("src")).unwrap();
    std::fs::write(
        crate_root.join("Cargo.toml"),
        "[package]\nname = 'worth-query-execution'\nversion = '0.1.0'\nedition = '2021'\n",
    )
    .unwrap();
    std::fs::write(crate_root.join("src/lib.rs"), source).unwrap();
    let governed =
        super::super::workspace_crates::discover_workspace_crates(&root, "workspaces/worth-query")
            .unwrap()
            .remove(0);
    let graph = super::super::crate_modules::parse_crate_modules(&governed).unwrap();
    let diagnostics = super::enforce(&governed, &graph);
    // This fixture owns this fresh leaf beneath the build's tmp directory.
    let canonical = root.canonicalize().unwrap();
    assert!(canonical.starts_with(root.parent().unwrap().canonicalize().unwrap()));
    std::fs::remove_dir_all(canonical).unwrap();
    diagnostics
}

#[test]
fn debug_scope_and_generic_key_holes_each_report_one_diagnostic() {
    for source in [
        include_str!("../../../../tests/fixtures/canonical_order_rules/debug_alias_scope.rs"),
        include_str!("../../../../tests/fixtures/canonical_order_rules/debug_generic_key.rs"),
    ] {
        let diagnostics = debug_diagnostics(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code(),
            crate::diagnostics::DiagnosticCode::Bc7009CanonicalOrder
        );
    }
}

#[test]
fn debug_alias_imported_from_a_sibling_reports_one_diagnostic() {
    let diagnostics = debug_diagnostics(include_str!(
        "../../../../tests/fixtures/canonical_order_rules/debug_imported_alias.rs"
    ));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code(),
        crate::diagnostics::DiagnosticCode::Bc7009CanonicalOrder
    );
}
