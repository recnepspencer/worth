#[test]
fn debug_scope_and_generic_key_holes_each_report_one_diagnostic() {
    use super::super::crate_modules::ModuleNode;
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/canonical_order");
    let governed =
        super::super::workspace_crates::discover_workspace_crates(&root, "workspaces/worth-query")
            .unwrap()
            .remove(0);
    let mut counts = Vec::new();
    for source in [
        include_str!("../../../../tests/fixtures/canonical_order_rules/debug_alias_scope.rs"),
        include_str!("../../../../tests/fixtures/canonical_order_rules/debug_generic_key.rs"),
    ] {
        let mut graph = super::super::crate_modules::parse_crate_modules(&governed).unwrap();
        graph.modules.clear();
        let items = crate::source_syntax::parse_file(source).unwrap().items;
        let mut root_items = Vec::new();
        for item in items {
            if let syn::Item::Mod(module) = item {
                graph.modules.insert(
                    vec![module.ident.to_string()],
                    ModuleNode {
                        relative_source: "src/lib.rs".into(),
                        public_from_parent: false,
                        attributes: Vec::new(),
                        items: module.content.unwrap().1,
                    },
                );
            } else {
                root_items.push(item);
            }
        }
        graph.modules.insert(
            Vec::new(),
            ModuleNode {
                relative_source: "src/lib.rs".into(),
                public_from_parent: false,
                attributes: Vec::new(),
                items: root_items,
            },
        );
        let diagnostics = super::enforce(&governed, &graph);
        counts.push(diagnostics.len());
        assert!(diagnostics
            .iter()
            .all(|d| d.code() == crate::diagnostics::DiagnosticCode::Bc7009CanonicalOrder));
    }
    assert_eq!(counts, [1, 1]);
}
