use super::super::crate_modules::{ModuleGraph, ModuleNode};
use super::check_graphs;
use crate::config::{RequestConstructorDenialConfig, Road1Config};
use std::collections::BTreeMap;

fn rule(name: &str) -> RequestConstructorDenialConfig {
    let config: Road1Config =
        toml::from_str(include_str!("../../../../config/road1.toml")).unwrap();
    config
        .request_constructor_denials
        .into_iter()
        .find(|rule| rule.crate_root == format!("crates/{name}"))
        .expect("the required crate's constructor fence")
}

fn graph(source: &str, host: Option<&str>) -> ModuleGraph {
    let mut modules = BTreeMap::new();
    modules.insert(
        Vec::new(),
        ModuleNode {
            relative_source: "src/lib.rs".into(),
            public_from_parent: false,
            attributes: Vec::new(),
            items: crate::source_syntax::parse_file(source).unwrap().items,
        },
    );
    if let Some(host) = host {
        modules.insert(vec!["host".into()], ModuleNode {
            relative_source: host.into(), public_from_parent: false, attributes: Vec::new(),
            items: crate::source_syntax::parse_file(
                "fn open(policy: Policy) -> SerialRequest { SerialRequest::from_policy(&policy, cancel, None) }"
            ).unwrap().items,
        });
    }
    ModuleGraph { modules }
}

fn rejects_mint(name: &str) {
    let rule = rule(name);
    let host = rule.host_entry.as_deref();
    let allowed = graph(
        "fn seam(request: ExecutionRequest) { consume(request); }",
        host,
    );
    assert!(check_graphs(&[allowed], &rule).is_empty());
    let mutant = graph(
        "fn seam(request: ExecutionRequest) { let minted = SerialRequest::from_policy(&policy, cancel, None); consume(minted); }", host,
    );
    let found = check_graphs(&[mutant], &rule);
    assert!(
        found
            .iter()
            .any(|denial| denial.subject().contains("src/lib.rs")
                && denial.message().contains("SerialRequest")),
        "{name} admitted a Query-path mint: {found:?}"
    );
}

#[test]
fn signal_constructor_fence_has_a_positive_control() {
    rejects_mint("worth-signal");
}
#[test]
fn bridge_constructor_fence_has_a_positive_control() {
    rejects_mint("worth-runtime-bridge");
}
#[test]
fn world_constructor_fence_has_a_positive_control() {
    rejects_mint("worth-runtime-world");
}

#[test]
fn aliases_constructor_values_macros_and_raw_names_cannot_hide_mints() {
    let rule = rule("worth-runtime-world");
    for source in [
        "use worth_execution::SerialRequest as Backing; fn seam() { Backing::from_policy(&p, c, None); }",
        "type Backing = worth_execution::SerialRequest;",
        "fn seam() { let mint = SerialRequest::from_memory; }",
        "fn seam() { p.serial_request(c, None); }",
        "macro_rules! mint { () => { SerialRequest::from_policy(&p, c, None) } }",
        "fn seam() { r#SerialRequest::from_policy(&p, c, None); }",
    ] {
        assert!(!check_graphs(&[graph(source, None)], &rule).is_empty(), "accepted {source}");
    }
}

#[test]
fn test_only_items_are_skipped_but_feature_selected_mints_are_production() {
    let rule = rule("worth-runtime-world");
    let source = r#"#[cfg(test)] mod tests { fn open() { SerialRequest::from_policy(&p,c,None); } }
        impl Owner { #[cfg(test)] fn open() { p.serial_request(c,None); } }
        const NOTE: &str = "SerialRequest serial_request";"#;
    assert!(check_graphs(&[graph(source, None)], &rule).is_empty());
    assert!(!check_graphs(
        &[graph(
            "#[cfg(feature = \"host\")] fn open() { SerialRequest::from_policy(&p,c,None); }",
            None,
        )],
        &rule
    )
    .is_empty());
}

#[test]
fn a_missing_declared_host_is_stale_and_an_unreadable_crate_fails_closed() {
    let rule = rule("worth-signal");
    let found = check_graphs(&[graph("fn seam(r: ExecutionRequest) {}", None)], &rule);
    assert_eq!(found.len(), 1);
    assert!(found[0].message().contains("stale allowance"));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let found = super::enforce_request_constructor_denials(root, &[rule]);
    assert_eq!(found.len(), 1);
    assert!(found[0].message().contains("could not be read"));
}
