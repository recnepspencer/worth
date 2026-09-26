use syn::visit::Visit;

use super::{
    diagnostics_for_source, validate_source_owner_isolations, OwnedItemCollector, OwnedItems,
};
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
        methods: std::collections::BTreeSet::new(),
        scoped_values: std::collections::BTreeMap::new(),
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
        "use crate::primary_graph as pg; use pg::workflow::definition;",
        "use crate::primary_graph as pg; fn f() { pg::workflow::start(); }",
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

/// What `rule()` isolates when the owner declares exactly `owner_source`.
fn collected(owner_source: &str) -> OwnedItems {
    let mut owned = OwnedItems::default();
    let mut declared = std::collections::BTreeSet::new();
    let mut methods = Vec::new();
    OwnedItemCollector {
        owned: &mut owned,
        declared: &mut declared,
        methods: &mut methods,
        module: vec!["owner".into(), "lane".into()],
    }
    .visit_file(&syn::parse_file(owner_source).unwrap());
    owned.add_foreign_type_methods(methods, &declared);
    owned
}

fn findings_against(owned: &OwnedItems, source: &str) -> Vec<String> {
    diagnostics_for_source("guarded/run.rs", source, owned, &rule())
        .into_iter()
        .map(|diagnostic| diagnostic.message().to_owned())
        .collect()
}

#[test]
fn a_method_the_owner_adds_to_a_foreign_type_is_refused_when_called_or_named() {
    let owned = collected(
        "pub struct WorkflowDefinition;          impl WorthQuerySelectedProductOperation {              pub(in crate::primary_graph) fn prepare_workflow_publication(&self) {}              fn private_step(&self) {}          }          impl WorkflowDefinition { pub fn new() -> Self { Self } }          struct PrivateEntity; impl PrivateEntity { pub(crate) fn text(&self) {} }          pub(super) fn encode() {}",
    );
    for source in [
        "fn f(product: &Product) { product.prepare_workflow_publication(); }",
        "fn f(product: &Product) {              WorthQuerySelectedProductOperation::prepare_workflow_publication(product); }",
    ] {
        let found = findings_against(&owned, source);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains("`prepare_workflow_publication`"), "{found:?}");
    }
    for legal in [
        "fn f(product: &Product) { product.private_step(); }",
        "fn f() { let _ = Run::new(); }",
        "fn f(material: &mut Material) { material.text(); material.encode(); }",
    ] {
        assert!(findings_against(&owned, legal).is_empty(), "{legal}");
    }
}

#[test]
fn test_only_owner_items_do_not_bind_guarded_code() {
    let owned = collected(
        "struct Fixture; #[cfg(test)] pub(crate) struct Harness;          #[cfg(test)] mod tests { pub(crate) fn seeded() {} } pub struct Kernel;",
    );
    assert!(owned.types.contains("Kernel"));
    assert!(findings_against(
        &owned,
        "struct Fixture; struct Harness; fn f() { crate::support::seeded(); }"
    )
    .is_empty());
}

/// A value visible only inside a module binds guarded code inside that
/// module and nowhere else; a shared name elsewhere is a different item.
#[test]
fn a_restricted_owner_value_binds_only_inside_its_scope() {
    let owned = collected(
        "pub(super) fn prepare() {} pub(in crate::primary_graph) fn exact() {}          pub(crate) fn settle() {}",
    );
    let outside = "crate/src/managed_run/run.rs";
    let inside = "crate/src/primary_graph/conditional_operation.rs";
    let source = "fn f() { Pending::prepare(); MappingSelector::exact(); Run::settle(); }";
    let reached = |path| {
        diagnostics_for_source(path, source, &owned, &rule())
            .into_iter()
            .map(|diagnostic| diagnostic.message().to_owned())
            .collect::<Vec<_>>()
    };
    let found = reached(outside);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`settle`"), "{found:?}");
    let found = reached(inside);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found.iter().any(|message| message.contains("`exact`")),
        "{found:?}"
    );
    let owner = "crate/src/owner/sibling.rs";
    assert!(reached(owner)
        .iter()
        .any(|message| message.contains("`prepare`")));
}

#[test]
fn test_only_items_of_every_kind_bind_nothing() {
    let owned = collected(
        "#[cfg(test)] pub const SEED: u8 = 0; #[cfg(test)] pub static CELL: u8 = 0;          #[cfg(test)] pub trait Probe {} #[cfg(test)] pub type Alias = u8;          #[cfg(test)] #[macro_export] macro_rules! fixture { () => {} }",
    );
    assert!(
        owned.types.is_empty() && owned.values.is_empty(),
        "test-only items were collected"
    );
}

/// Only a `#[cfg(test)]` module declaration exempts a file, however the file
/// is named; production files named `tests.rs` still bind guarded code.
#[test]
fn a_cfg_test_module_declaration_exempts_its_files_and_nothing_else() {
    let workspace =
        std::env::temp_dir().join(format!("boundary-owner-tests-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    for directory in [
        "owner/state/tests",
        "owner/audit",
        "owner/progress",
        "guarded",
    ] {
        std::fs::create_dir_all(workspace.join(directory)).unwrap();
    }
    for (file, source) in [
        (
            "owner/state.rs",
            "#[cfg(test)] mod tests; #[cfg(test)] #[path = \"state/checks.rs\"] mod checks;              mod audit_hook; pub struct Kernel;",
        ),
        ("owner/state/tests.rs", "pub struct Harness;"),
        ("owner/state/tests/nested.rs", "pub struct NestedHarness;"),
        ("owner/state/checks.rs", "pub struct Checks;"),
        ("owner/audit.rs", "pub mod tests;"),
        ("owner/audit/tests.rs", "pub struct AuditRecord;"),
        ("owner/progress.rs", "#[cfg(test)] mod scaling;"),
        ("owner/progress/scaling.rs", "pub struct ScalingFixture;"),
        (
            "guarded/run.rs",
            "fn f(_: Kernel, _: Harness, _: NestedHarness, _: Checks,                  _: AuditRecord, _: ScalingFixture) {}",
        ),
    ] {
        std::fs::write(workspace.join(file), source).unwrap();
    }
    let found = validate_source_owner_isolations(&workspace, &[rule()]);
    std::fs::remove_dir_all(&workspace).unwrap();
    let messages = found.iter().map(|d| d.message()).collect::<Vec<_>>();
    assert_eq!(found.len(), 2, "{messages:?}");
    for reached in ["`Kernel`", "`AuditRecord`"] {
        assert!(
            messages.iter().any(|m| m.contains(reached)),
            "{reached}: {messages:?}"
        );
    }
}

#[test]
fn a_relative_path_inside_an_inline_module_resolves_against_that_module() {
    let sibling = "crate/src/primary_graph/conditional_operation.rs";
    let found = findings_at(
        sibling,
        "mod helpers { use super::super::workflow::advance; }",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`primary_graph::workflow`"), "{found:?}");
    assert!(findings_at(sibling, "mod workflow { } use self::workflow::Local;").is_empty());
}

/// A bare name is not traced to the owner, so a glob import may not reach
/// outside the guarded roots, where a facade could re-export owner values.
#[test]
fn a_glob_import_from_outside_the_guarded_roots_is_refused() {
    for (source, reach) in [
        ("use crate::facade::*;", "crate::facade::*"),
        ("use super::super::*;", "super::super::*"),
        (
            "mod inner { use crate::{guarded::run, facade::*}; }",
            "crate::facade::*",
        ),
        ("use crate::facade as f; use f::*;", "f::*"),
        ("use super::super as up; use up::*;", "up::*"),
        ("use crate::{facade}; use facade::*;", "facade::*"),
        ("use crate::facade::{self as f}; use f::*;", "f::*"),
        (
            "use crate::facade as f; use f::inner as g; use g::*;",
            "g::*",
        ),
        ("use crate::*;", "crate::*"),
    ] {
        let found = findings(source);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains(&format!("`{reach}`")), "{found:?}");
    }
    for legal in [
        "use super::*;",
        "use self::local::*;",
        "use crate::guarded::support::*;",
        "use worth_foundational::facade::*;",
        "mod tests { use super::*; }",
        "use crate::guarded as g; use g::support::*;",
        "use std::collections as c; use c::*;",
    ] {
        assert!(findings(legal).is_empty(), "{legal}");
    }
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
    let lanes = "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt";
    for lane in [
        "workflow_definition_program",
        "workflow_instance_observation",
        "workflow_instance_program",
        "workflow_proposal_program",
        "workflow_transition_program",
    ] {
        for owned in [format!("{lanes}/{lane}"), format!("{lanes}/{lane}.rs")] {
            assert!(rule.owner_roots.contains(&owned), "{owned} is not isolated");
        }
    }
}

/// A workflow application lane is part of the isolated owner: guarded code
/// naming a lane's outcome type is refused like a kernel item.
#[test]
fn guarded_code_naming_a_workflow_lane_item_is_refused() {
    let owned = collected("pub enum WorkflowInstanceStartOutcome { Started }");
    let found = findings_against(
        &owned,
        "fn settle(outcome: crate::primary_graph::WorkflowInstanceStartOutcome) {}",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`WorkflowInstanceStartOutcome`"),
        "{found:?}"
    );
}
