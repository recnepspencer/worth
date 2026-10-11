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
fn production_raw_construction_is_refused() {
    for source in [
        "fn f() { let raw = ExecutionAllocationDenialKind::Layout; }",
        "fn f(k: Other) { matches!(ExecutionAllocationDenialKind::Layout, _); }",
        "fn f(k: ExecutionAllocationDenialKind) { ExecutionAllocationDenialKind::classify(k); }",
    ] {
        assert!(refused(source), "accepted {source}");
    }
}
#[test]
fn test_only_raw_access_is_accepted() {
    for source in [
        "#[cfg(test)] fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }",
        "#[cfg(test)] mod tests { fn f(k: ExecutionAllocationDenialKind) { matches!(k, ExecutionAllocationDenialKind::Cancelled); } }",
    ] { assert!(!refused(source), "rejected {source}"); }
}
#[test]
fn raw_if_let_pattern_is_refused() {
    for source in [
        "fn f(k: ExecutionAllocationDenialKind) { if let ExecutionAllocationDenialKind::Cancelled = k {} }",
        "fn f(k: ExecutionAllocationDenialKind) { let ExecutionAllocationDenialKind::Lease(lease) = k else { return }; }",
        "fn f(k: ExecutionAllocationDenialKind) { while let ExecutionAllocationDenialKind::Cancelled = k {} }",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn imported_bare_variants_are_refused() {
    for source in [
        "use worth_execution::ExecutionAllocationDenialKind::*; fn f(k: ExecutionAllocationDenialKind) { match k { Cancelled => (), Lease(_) => (), _ => () } }",
        "use worth_execution::ExecutionAllocationDenialKind::{Cancelled, Lease}; fn f(k: ExecutionAllocationDenialKind) { if let Lease(_) = k {} }",
        "use worth_execution::ExecutionAllocationDenialKind::Cancelled; fn f(k: ExecutionAllocationDenialKind) { matches!(k, Cancelled); }",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn raw_type_alias_is_refused() {
    for source in [
        "type K = worth_execution::ExecutionAllocationDenialKind; fn f(k: K) { matches!(k, K::Cancelled); }",
        "trait T { type K = worth_execution::ExecutionAllocationDenialKind; }",
        "impl T for Other { type K = worth_execution::ExecutionAllocationDenialKind; }",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn foreign_trait_impl_is_refused() {
    for source in [
        "impl LocalTrait for worth_execution::ExecutionAllocationDenialKind { fn stopped(self) -> bool { matches!(self, Self::Cancelled) } }",
        "impl LocalTrait for &worth_execution::ExecutionAllocationDenialKind {}",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn type_named_alone_in_a_signature_is_accepted() {
    assert!(!refused("pub use worth_execution::ExecutionAllocationDenialKind; struct Holder { kind: ExecutionAllocationDenialKind } fn forward(kind: ExecutionAllocationDenialKind) -> ExecutionAllocationDenialKind { kind } type OtherAlias = Other; impl LocalTrait for Other {}"));
}
#[test]
fn raw_access_inside_macros_is_refused() {
    for source in [
        "fn f(k: ExecutionAllocationDenialKind) { let v = vec![match k { ExecutionAllocationDenialKind::Cancelled => 1, _ => 0 }]; }",
        "macro_rules! alias { () => { type K = ExecutionAllocationDenialKind; }; }",
        "macro_rules! extension { () => { impl LocalTrait for ExecutionAllocationDenialKind { fn stopped(self) -> bool { matches!(self, Self::Cancelled) } } }; }",
    ] { assert!(refused(source), "accepted {source}"); }
}
#[test]
fn only_the_execution_owner_can_match_raw_patterns() {
    let graph = graph("fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }");
    assert!(check_graphs("crates/worth-execution", &[graph]).is_empty());
}
