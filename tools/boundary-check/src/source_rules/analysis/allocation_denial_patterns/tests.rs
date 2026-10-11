use super::super::crate_modules::{ModuleGraph, ModuleNode};
use super::check_graphs;
use std::collections::BTreeMap;

fn graph(source: &str) -> ModuleGraph {
    ModuleGraph {
        modules: BTreeMap::from([(
            Vec::new(),
            ModuleNode {
                relative_source: "src/lib.rs".to_owned(),
                public_from_parent: false,
                attributes: Vec::new(),
                items: crate::source_syntax::parse_file(source).unwrap().items,
            },
        )]),
    }
}
fn refused(source: &str) -> bool {
    !check_graphs("crates/consumer", &[graph(source)]).is_empty()
}

#[test]
fn raw_match_arm_is_refused() {
    assert!(refused("fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }"));
}
#[test]
fn classifier_call_is_accepted() {
    assert!(!refused("fn f(k: ExecutionAllocationDenialKind) { match k.classify() { ExecutionAllocationDenialClass::Request(cause) => (), ExecutionAllocationDenialClass::Physical(kind) => () } }"));
}
#[test]
fn raw_construction_in_a_test_file_is_accepted() {
    let mut graph = graph("#[cfg(test)] mod tests;");
    graph.modules.insert(
        vec!["tests".into()],
        ModuleNode {
            relative_source: "src/tests.rs".into(),
            public_from_parent: false,
            attributes: Vec::new(),
            items: crate::source_syntax::parse_file(
                "fn fixture() { let raw = ExecutionAllocationDenialKind::Layout; }",
            )
            .unwrap()
            .items,
        },
    );
    assert!(check_graphs("crates/consumer", &[graph]).is_empty());
}
#[test]
fn aliases_and_nested_matches_patterns_are_refused() {
    for source in [
        "use worth_execution::ExecutionAllocationDenialKind as Raw; fn f(k: Raw) { match k { Raw::Lease(_) => (), _ => () } }",
        "fn f(k: ExecutionAllocationDenialKind) { matches!(k, ExecutionAllocationDenialKind::Cancelled); }",
        "fn f(k: ExecutionAllocationDenialKind) { std::matches!(k, ExecutionAllocationDenialKind::Cancelled); }",
        "fn f(k: ExecutionAllocationDenialKind) { assert!(matches!(k, ExecutionAllocationDenialKind::Lease(_) | ExecutionAllocationDenialKind::Cancelled)); }",
        "fn f(k: ExecutionAllocationDenialKind) { use worth_execution::ExecutionAllocationDenialKind as Raw; matches!(k, Raw::Cancelled if true); }",
        "fn f(k: ExecutionAllocationDenialKind) { match k { r#ExecutionAllocationDenialKind::Layout => (), _ => () } }",
        "macro_rules! stopped { ($value:expr) => { matches!($value, ExecutionAllocationDenialKind::Cancelled) }; }",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn construction_and_test_only_patterns_are_accepted() {
    for source in [
        "fn f() { let raw = ExecutionAllocationDenialKind::Layout; }",
        "fn f(k: Other) { matches!(ExecutionAllocationDenialKind::Layout, _); }",
        "#[cfg(test)] fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }",
        "#[cfg(test)] mod tests { fn f(k: ExecutionAllocationDenialKind) { matches!(k, ExecutionAllocationDenialKind::Cancelled); } }",
    ] { assert!(!refused(source), "rejected {source}"); }
}
#[test]
fn only_the_execution_owner_can_match_raw_patterns() {
    let graph = graph("fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }");
    assert!(check_graphs("crates/worth-execution", &[graph]).is_empty());
}
