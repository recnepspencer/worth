use super::{diagnostics_for_source, validate_source_owner_isolations, OwnedItems};
use crate::config::SourceOwnerIsolationConfig;

fn rule() -> SourceOwnerIsolationConfig {
    SourceOwnerIsolationConfig {
        owner_roots: vec!["owner".into()],
        guarded_roots: vec!["guarded".into()],
        forbidden_paths: vec![vec!["primary_graph".into(), "workflow".into()]],
        guidance: "the guarded owner never reaches the workflow kernel".into(),
    }
}

fn owned() -> OwnedItems {
    OwnedItems {
        types: ["PublishedWorkflowDefinitionRef".to_owned()].into(),
        values: [
            "advance_workflow_instance".to_owned(),
            "MAX_BACK_EDGES".to_owned(),
        ]
        .into(),
    }
}

fn findings(source: &str) -> Vec<String> {
    findings_at("guarded/run.rs", source)
}

fn findings_at(path: &str, source: &str) -> Vec<String> {
    diagnostics_for_source(path, source, &owned(), &rule())
        .into_iter()
        .map(|diagnostic| diagnostic.message().to_owned())
        .collect()
}

#[test]
fn a_path_through_the_isolated_owner_is_refused_in_every_position() {
    for source in [
        "use crate::primary_graph::workflow::definition;",
        "use crate::primary_graph::{workflow::definition, root};",
        "use crate::primary_graph::workflow;",
        "use crate::primary_graph::workflow::*;",
        "fn f() { crate::primary_graph::workflow::start(); }",
        "fn f() { format!(\"{}\", crate::primary_graph::workflow::NAME); }",
    ] {
        let found = findings(source);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains("`primary_graph::workflow`"), "{found:?}");
    }
}

#[test]
fn an_owned_type_reexported_through_a_parent_is_still_refused() {
    let found = findings("use crate::primary_graph::PublishedWorkflowDefinitionRef;");
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("`PublishedWorkflowDefinitionRef`"));
}

#[test]
fn owned_functions_and_constants_reexported_through_a_parent_are_refused() {
    for (source, reference) in [
        (
            "fn f() { crate::facade::advance_workflow_instance(); }",
            "advance_workflow_instance",
        ),
        (
            "use crate::facade::{MAX_BACK_EDGES, root};",
            "MAX_BACK_EDGES",
        ),
    ] {
        let found = findings(source);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains(&format!("`{reference}`")), "{found:?}");
    }
}

#[test]
fn a_local_binding_sharing_an_owned_function_name_is_legal() {
    assert!(
        findings("fn f(advance_workflow_instance: u8) -> u8 { advance_workflow_instance }")
            .is_empty()
    );
}

#[test]
fn a_relative_path_resolves_against_the_guarded_file_module() {
    let sibling = "crate/src/primary_graph/conditional_operation.rs";
    let nested = "crate/src/primary_graph/conditional_operation/admission.rs";
    for (path, source) in [
        (sibling, "use super::workflow::definition;"),
        (sibling, "fn f() { super::workflow::start(); }"),
        (nested, "use super::super::workflow::start;"),
        (nested, "use crate::primary_graph::workflow;"),
    ] {
        let found = findings_at(path, source);
        assert_eq!(found.len(), 1, "{path}: {source}: {found:?}");
        assert!(found[0].contains("`primary_graph::workflow`"), "{found:?}");
    }
    let own_module = "crate/src/managed_run/workflow_stage.rs";
    assert!(findings_at(own_module, "use super::workflow::WorkflowRun;").is_empty());
    assert!(findings_at(nested, "use super::workflow_gate::Gate;").is_empty());
}

#[test]
fn a_same_named_module_of_the_guarded_owner_is_legal() {
    assert!(findings(
        "mod workflow; use super::workflow::WorthQueryWorkflowRunTerminal; \
         use crate::primary_graph::root::Workflowish;"
    )
    .is_empty());
}

#[test]
fn configured_isolation_collects_owned_types_from_the_owner_root() {
    let workspace =
        std::env::temp_dir().join(format!("boundary-owner-isolation-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(workspace.join("owner/instance")).unwrap();
    std::fs::create_dir_all(workspace.join("guarded")).unwrap();
    std::fs::write(
        workspace.join("owner/instance/state.rs"),
        "pub struct WorkflowInstanceState; pub enum WorkflowProgressOutcome {}          pub(crate) fn advance_workflow_instance() {} fn private_helper() {}",
    )
    .unwrap();
    std::fs::write(
        workspace.join("guarded/run.rs"),
        "fn f(_: crate::facade::WorkflowProgressOutcome) { crate::facade::private_helper();          crate::facade::advance_workflow_instance(); }",
    )
    .unwrap();
    let found = validate_source_owner_isolations(&workspace, &[rule()]);
    std::fs::remove_dir_all(&workspace).unwrap();
    let messages = found.iter().map(|d| d.message()).collect::<Vec<_>>();
    assert_eq!(found.len(), 2, "{messages:?}");
    assert!(messages
        .iter()
        .any(|m| m.contains("`WorkflowProgressOutcome`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("`advance_workflow_instance`")));
}

#[test]
fn road1_isolates_managed_run_and_conditional_operation_from_the_workflow_kernel() {
    let config: crate::config::Road1Config =
        toml::from_str(include_str!("../../config/road1.toml")).unwrap();
    let kernel = "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/workflow";
    let rule = config
        .source_owner_isolations
        .iter()
        .find(|rule| rule.owner_roots.iter().any(|root| root == kernel))
        .expect("road1 isolates the workflow kernel");
    for guarded in [
        "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/managed_run",
        "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/conditional_operation",
        "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/conditional_operation.rs",
    ] {
        assert!(rule.guarded_roots.iter().any(|root| root == guarded), "{guarded}");
    }
}
