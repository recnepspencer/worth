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

fn assert_raw_kind_diagnostics(source: &str, count: usize) {
    let diagnostics = check_graphs("crates/consumer", &[graph(source)]);
    assert_eq!(diagnostics.len(), count, "{diagnostics:#?}");
    for diagnostic in diagnostics {
        assert_eq!(
            diagnostic.code(),
            crate::diagnostics::DiagnosticCode::Bc7010AllocationDenialPattern
        );
    }
}

#[test]
fn raw_match_arm_is_refused() {
    assert_raw_kind_diagnostics("fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }", 1);
}
#[test]
fn classifier_call_is_accepted() {
    assert_raw_kind_diagnostics("fn f(k: ExecutionAllocationDenialKind) { match k.classify() { ExecutionAllocationDenialClass::Request(cause) => (), ExecutionAllocationDenialClass::Physical(kind) => () } }", 0);
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
    ] { assert_raw_kind_diagnostics(source, 1); }
}
#[test]
fn production_raw_construction_is_refused() {
    for source in [
        "fn f() { let raw = ExecutionAllocationDenialKind::Layout; }",
        "fn f(k: Other) { matches!(ExecutionAllocationDenialKind::Layout, _); }",
        "fn f(k: ExecutionAllocationDenialKind) { ExecutionAllocationDenialKind::classify(k); }",
    ] {
        assert_raw_kind_diagnostics(source, 1);
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
    ] { assert_raw_kind_diagnostics(source, 1); }
}
#[test]
fn imported_bare_variants_are_refused() {
    for source in [
        "use worth_execution::ExecutionAllocationDenialKind::*; fn f(k: ExecutionAllocationDenialKind) { match k { Cancelled => (), Lease(_) => (), _ => () } }",
        "use worth_execution::ExecutionAllocationDenialKind::{Cancelled, Lease}; fn f(k: ExecutionAllocationDenialKind) { if let Lease(_) = k {} }",
        "use worth_execution::ExecutionAllocationDenialKind::Cancelled; fn f(k: ExecutionAllocationDenialKind) { matches!(k, Cancelled); }",
    ] { assert_raw_kind_diagnostics(source, 1); }
}
#[test]
fn raw_type_alias_is_refused() {
    for source in [
        "type K = worth_execution::ExecutionAllocationDenialKind; fn f(k: K) { matches!(k, K::Cancelled); }",
        "trait T { type K = worth_execution::ExecutionAllocationDenialKind; }",
        "impl T for Other { type K = worth_execution::ExecutionAllocationDenialKind; }",
    ] { assert_raw_kind_diagnostics(source, if source.starts_with("type K") { 2 } else { 1 }); }
}
#[test]
fn foreign_trait_impl_is_refused() {
    for source in [
        "impl LocalTrait for worth_execution::ExecutionAllocationDenialKind { fn stopped(self) -> bool { matches!(self, Self::Cancelled) } }",
        "impl LocalTrait for &worth_execution::ExecutionAllocationDenialKind {}",
    ] { assert_raw_kind_diagnostics(source, 1); }
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
    ] { assert_raw_kind_diagnostics(source, 1); }
}
#[test]
fn only_the_execution_owner_can_match_raw_patterns() {
    let graph = graph("fn f(k: ExecutionAllocationDenialKind) { match k { ExecutionAllocationDenialKind::Cancelled => (), _ => () } }");
    assert!(check_graphs("crates/worth-execution", &[graph]).is_empty());
}

#[test]
fn qualified_self_pattern_is_refused() {
    assert_raw_kind_diagnostics("fn f(k: ExecutionAllocationDenialKind) { match k { <ExecutionAllocationDenialKind>::Cancelled => (), _ => () } }", 1);
}

#[test]
fn qualified_self_expression_is_refused() {
    assert_raw_kind_diagnostics(
        "fn f() { let raw = <ExecutionAllocationDenialKind>::Cancelled; }",
        1,
    );
}

#[test]
fn qualified_self_trait_projection_is_refused() {
    assert_raw_kind_diagnostics(
        "impl T for <ExecutionAllocationDenialKind as Id>::Out {}",
        1,
    );
}

#[test]
fn generic_argument_names_only_the_type() {
    assert!(!refused(
        "struct Holder { kinds: Vec<ExecutionAllocationDenialKind> }"
    ));
}

#[test]
fn alias_of_a_container_does_not_alias_the_raw_kind() {
    for source in [
        "type K = Vec<ExecutionAllocationDenialKind>;",
        "trait T { type K = Vec<ExecutionAllocationDenialKind>; }",
        "impl T for Other { type K = Vec<ExecutionAllocationDenialKind>; }",
    ] {
        assert!(!refused(source), "rejected {source}");
    }
}

#[test]
fn a_doubly_nested_macro_reports_one_raw_access_once() {
    let source = r#"
        macro_rules! outer {
            () => {
                macro_rules! inner {
                    () => {
                        fn generated() {
                            let raw = ExecutionAllocationDenialKind::Cancelled;
                        }
                    };
                }
            };
        }
    "#;

    assert_raw_kind_diagnostics(source, 1);
}

#[test]
fn a_variant_import_in_an_inline_module_inside_a_macro_is_refused_once() {
    assert_raw_kind_diagnostics("macro_rules! outer { () => { mod inner { use worth_execution::ExecutionAllocationDenialKind::Cancelled; } }; }", 1);
}

#[test]
fn a_variant_import_in_a_nested_inline_module_is_refused() {
    assert_raw_kind_diagnostics("mod outer { mod inner { use worth_execution::ExecutionAllocationDenialKind::Cancelled; } }", 1);
}

#[test]
fn a_transparent_alias_cannot_read_a_raw_variant() {
    assert_raw_kind_diagnostics(
        "type Id<T> = T; fn f() { let raw = Id::<ExecutionAllocationDenialKind>::Cancelled; }",
        1,
    );
}

#[test]
fn an_alias_of_a_transparent_alias_is_raw_at_both_sites() {
    assert_raw_kind_diagnostics("type Id<T> = T; type K = Id<ExecutionAllocationDenialKind>; fn f() { let raw = K::Cancelled; }", 2);
}

#[test]
fn a_nested_qualified_self_cannot_recover_a_raw_variant() {
    assert_raw_kind_diagnostics("fn f() { let raw = <<Vec<ExecutionAllocationDenialKind> as IntoIterator>::Item>::Cancelled; }", 1);
}

#[test]
fn transparent_alias_chains_preserve_the_selected_parameter() {
    assert_raw_kind_diagnostics("type Id<T> = T; type Through<U> = Id<U>; type Again<V> = Through<Id<V>>; fn f() { let raw = Again::<ExecutionAllocationDenialKind>::Cancelled; }", 1);
}

#[test]
fn same_named_container_and_identity_aliases_keep_their_scopes() {
    assert_raw_kind_diagnostics("mod identity { type Id<T> = T; fn f() { let raw = Id::<ExecutionAllocationDenialKind>::Cancelled; } } mod container { type Id<T> = Vec<T>; type Kinds = Id<ExecutionAllocationDenialKind>; struct Holder { kinds: Kinds } }", 1);
}

#[test]
fn a_transparent_alias_in_another_file_is_still_raw() {
    let mut source = graph("mod aliases; use aliases::Id; fn f() { let raw = Id::<ExecutionAllocationDenialKind>::Cancelled; }");
    source.modules.insert(
        vec!["aliases".into()],
        ModuleNode {
            relative_source: "src/aliases.rs".into(),
            public_from_parent: false,
            attributes: Vec::new(),
            items: crate::source_syntax::parse_file("pub type Id<T> = T;")
                .unwrap()
                .items,
        },
    );
    let diagnostics = check_graphs("crates/consumer", &[source]);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code(),
        crate::diagnostics::DiagnosticCode::Bc7010AllocationDenialPattern
    );
}

#[test]
fn a_literal_alias_read_inside_a_macro_is_refused_once() {
    assert_raw_kind_diagnostics("type Id<T> = T; fn f() { matches!(kind, Id::<ExecutionAllocationDenialKind>::Cancelled); }", 1);
}

#[test]
fn a_literal_alias_read_in_guarded_or_token_macro_syntax_is_refused_once() {
    for source in [
        "type Id<T> = T; fn f() { matches!(kind, Id::<ExecutionAllocationDenialKind>::Cancelled if guard); }",
        "type Id<T> = T; fn f() { tokens!(Id::<ExecutionAllocationDenialKind>::Cancelled => value); }",
    ] { assert_raw_kind_diagnostics(source, 1); }
}

#[test]
fn qualified_self_tuple_and_struct_patterns_are_reads() {
    for source in [
        "fn f(k: ExecutionAllocationDenialKind) { match k { <ExecutionAllocationDenialKind>::Lease(_) => (), _ => () } }",
        "fn f(k: ExecutionAllocationDenialKind) { match k { <ExecutionAllocationDenialKind>::Cancelled {} => (), _ => () } }",
    ] { assert_raw_kind_diagnostics(source, 1); }
}

#[test]
fn a_multiline_qualified_self_read_is_one_diagnostic() {
    assert_raw_kind_diagnostics("fn f() { let raw = <\n<Vec<ExecutionAllocationDenialKind> as IntoIterator>::Item\n>::Cancelled; }", 1);
}

#[test]
fn paths_past_types_mentioning_raw_are_reads() {
    for source in [
        "type Item<C> = <C as IntoIterator>::Item; fn f() { Item::<Vec<ExecutionAllocationDenialKind>>::Cancelled; }",
        "type Kinds = Vec<ExecutionAllocationDenialKind>; fn f() { Kinds::new(); }",
        "type Kinds = Vec<ExecutionAllocationDenialKind>; fn f() { <Kinds>::new(); }",
        "impl T for Item<Vec<ExecutionAllocationDenialKind>>::Out {}",
        "fn f(k: Other) { match k { Item::<Vec<ExecutionAllocationDenialKind>>::Cancelled => (), _ => () } }",
    ] {
        assert_raw_kind_diagnostics(source, 1);
    }
}

#[test]
fn a_descendants_nominal_type_shadows_an_ancestor_alias() {
    assert_raw_kind_diagnostics("type K = ExecutionAllocationDenialKind; mod child { struct K; impl K { fn new() -> Self { Self } } fn f() { K::new(); } }", 1);
}

#[test]
fn an_unclosed_angle_in_a_macro_cannot_hide_a_raw_kind_path() {
    assert_raw_kind_diagnostics(
        "fn f() { tokens!(a < b; ExecutionAllocationDenialKind::Cancelled => value); }",
        1,
    );
}

#[test]
fn a_label_before_a_leading_path_in_a_macro_cannot_hide_a_raw_kind_path() {
    assert_raw_kind_diagnostics(
        "fn f() { tokens!(kind: ::worth_execution::ExecutionAllocationDenialKind::Cancelled => value); }",
        1,
    );
}

#[test]
fn an_unclosed_comparison_cannot_hide_a_generic_alias_read() {
    for source in [
        "type Id<T> = T; fn f() { tokens!(a < b, Id::<ExecutionAllocationDenialKind>::Cancelled => value); }",
        "type Id<T> = T; fn f() { tokens!(a < b, Id::<ExecutionAllocationDenialKind>::Cancelled: value); }",
    ] {
        assert_raw_kind_diagnostics(source, 1);
    }
}

#[test]
fn closed_angles_in_a_macro_cannot_hide_a_generic_alias_read() {
    for source in [
        "type Id<T> = T; fn f() { tokens!(a < b, Id::<ExecutionAllocationDenialKind>::Cancelled > c => value); }",
        "type Id<T> = T; fn f() { tokens!(a < b, c > Id::<ExecutionAllocationDenialKind>::Cancelled => value); }",
        "type Id<T> = T; fn f() { tokens!(Vec<u8> Id::<ExecutionAllocationDenialKind>::Cancelled => value); }",
    ] {
        assert_raw_kind_diagnostics(source, 1);
    }
}

#[test]
fn qualified_self_trait_projection_in_a_macro_is_refused() {
    // A literal raw ident followed by closing angles and :: cannot see this
    // projection: the trait name occurs between the raw ident and the member.
    assert_raw_kind_diagnostics(
        "fn f() { tokens!(<ExecutionAllocationDenialKind as Id>::Out => value); }",
        1,
    );
}
