//! Same-spelled names must resolve to their actual receiver, not the kernel.
use super::*;

fn collision_items() -> OwnedItems {
    collected("pub(crate) enum Command {} pub(crate) fn admit() {} pub(crate) fn encode() {}")
}

#[test]
fn standard_library_paths_and_aliases_do_not_reach_same_named_owner_types() {
    for source in [
        r#"fn f() { std::process::Command::new("court"); }"#,
        r#"use std::process::Command; fn f() { Command::new("court"); }"#,
        r#"use std::process as p; fn f() { p::Command::new("court"); }"#,
        r#"fn f() { choose!(std::process::Command); }"#,
    ] {
        assert!(
            findings_against(&collision_items(), source).is_empty(),
            "{source}"
        );
    }
    for source in [
        "use crate::facade::Command;",
        "fn f() { choose!(std crate::facade::Command); }",
        "fn f() { choose!(std Command); }",
        "fn f() { choose!(std:Command); }",
        "fn f() { use crate::facade as x; x::std::Command::new(); } fn g() { use std as x; }",
        "use crate::facade as std; fn f() { std::Command::new(); }",
        "mod std {} fn f() { std::Command::new(); }",
        "fn f() { std::collections::Vec::<crate::facade::Command>::new(); }",
        r#"fn f() { format!("{:?}", crate::facade::Command::new()); }"#,
    ] {
        assert!(
            !findings_against(&collision_items(), source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn declared_generic_receivers_do_not_reach_same_named_owner_free_functions() {
    for source in [
        "fn f<Field>() { Field::Binding::encode(); }",
        "struct Run<T>(T); impl<Field> Run<Field> { fn f() { Field::Binding::encode(); } }",
        "struct Run; impl Run { fn f<Field>() { Field::Binding::encode(); } }",
    ] {
        assert!(
            findings_against(&collision_items(), source).is_empty(),
            "{source}"
        );
    }
    for source in [
        "fn f<Field>() { Field::Binding::encode(); crate::facade::encode(); }",
        "fn f<Field>() { Field::Binding::encode(); } fn g() { Field::Binding::encode(); }",
        "fn f<Field>() { Field::Binding::<crate::facade::Command>::encode(); }",
        "fn f<Field>() { crate::primary_graph::workflow::encode(); }",
    ] {
        assert!(
            !findings_against(&collision_items(), source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn manifest_dependency_aliases_disambiguate_external_associated_functions() {
    let workspace = worth_harness::tooling::fixture_path::unique_path("isolation-external-paths");
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("Cargo.toml"),
        "[package]\nname='guarded'\nversion='0.1.0'\n[dependencies]\nbridge-alias={package='worth-runtime-bridge',version='1'}\n").unwrap();
    let external = super::super::path_resolution::external_crates(&workspace, "src/run.rs");
    assert!(external.contains("bridge_alias"));
    assert!(!external.contains("worth_runtime_bridge"));
    for (source, refused) in [
        ("use bridge_alias::facade::Candidate; fn f() { Candidate::admit(); }", false),
        ("use crate::facade::Candidate; fn f() { Candidate::admit(); }", true),
        ("use bridge_alias::facade::Candidate; fn f() { Candidate::admit::<crate::facade::Command>(); }", true),
        ("use crate::primary_graph::workflow as bridge_alias; fn f() { bridge_alias::admit(); }", true),
        ("use unknown::Candidate; fn f() { Candidate::admit(); }", true),
        ("extern crate self as bridge_alias; fn f() { bridge_alias::facade::Command::new(); }", true),
        ("extern crate bridge_alias as bridge; fn f() { bridge::facade::Candidate::admit(); }", false),
    ] {
        let found = super::super::diagnostics_for_syntax(
            "guarded/run.rs", crate::source_syntax::parse_file(source).map(std::rc::Rc::new),
            &collision_items(), &rule(), external.clone());
        assert_eq!(!found.is_empty(), refused, "{source}: {found:?}");
    }
    std::fs::write(workspace.join("Cargo.toml"), "not valid TOML").unwrap();
    assert!(
        !super::super::path_resolution::external_crates(&workspace, "src/run.rs")
            .contains("bridge_alias")
    );
}

#[test]
fn foreign_type_extensions_remain_refused_on_external_and_generic_paths() {
    let owned = collected("impl External { pub(crate) fn workflow_extension() {} }");
    for source in [
        "fn f() { std::External::workflow_extension(); }",
        "use std::External as E; fn f() { E::workflow_extension(); }",
        "fn f<T>() { T::workflow_extension(); }",
        "fn f(value: &External) { value.workflow_extension(); }",
    ] {
        let found = findings_against(&owned, source);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains("workflow_extension"));
    }
}
