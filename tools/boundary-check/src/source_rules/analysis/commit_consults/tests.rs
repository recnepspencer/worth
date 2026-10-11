use super::super::crate_modules::{ModuleGraph, ModuleNode};
use super::{check_graphs, OWNER};
use std::collections::BTreeMap;
fn graph(owner: &str, extra: Option<(&str, &str)>) -> ModuleGraph {
    let mut modules = BTreeMap::new();
    for (index, (source, code)) in std::iter::once((OWNER, owner)).chain(extra).enumerate() {
        modules.insert(
            vec![format!("part{index}")],
            ModuleNode {
                relative_source: source.into(),
                public_from_parent: false,
                attributes: Vec::new(),
                items: crate::source_syntax::parse_file(code).unwrap().items,
            },
        );
    }
    ModuleGraph { modules }
}
const VALID: &str = "fn consult_stage_control(request: &ExecutionRequest) { request.consult(); }";
#[test]
fn commit_consults_admit_only_the_named_stage_owner() {
    assert!(check_graphs(&[graph(
        VALID,
        Some((
            "src/mvcc/transaction/commit.rs",
            "fn commit(request: &ExecutionRequest) { consult_stage(request, Validation); }"
        ))
    )])
    .is_empty());
}
#[test]
fn commit_consults_reject_direct_calls_at_every_commit_owner() {
    for source in [
        "src/authority/commit/pipeline/execution.rs",
        "src/mvcc/transaction/commit.rs",
        "src/mvcc/validation/proposal_validation.rs",
        "src/mvcc/publication/port.rs",
    ] {
        let found = check_graphs(&[graph(
            VALID,
            Some((
                source,
                "fn bypass(request: &ExecutionRequest) { request.consult(); }",
            )),
        )]);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].message().contains("bypasses the commit stage"));
    }
}
#[test]
fn commit_consults_reject_aliases_macros_and_raw_identifiers() {
    for code in [
        "use worth_execution::ExecutionRequest::consult as hidden;",
        "fn bypass(request: &ExecutionRequest) { request.r#consult(); }",
        "fn bypass(request: &ExecutionRequest) { inspect!(request.consult()); }",
        "fn bypass() { let hidden = ExecutionRequest::consult; }",
    ] {
        assert!(
            !check_graphs(&[graph(VALID, Some(("src/mvcc/transaction/commit.rs", code)))])
                .is_empty(),
            "{code}"
        );
    }
}
#[test]
fn commit_consults_keep_the_owner_allowance_at_one_function_and_one_consult() {
    for owner in ["fn consult_stage_control(request: &ExecutionRequest) {}",
        "fn consult_stage_control(request: &ExecutionRequest) { request.consult(); request.consult(); }",
        "fn consult_stage_control(request: &ExecutionRequest) { request.consult(); } fn bypass(request: &ExecutionRequest) { request.consult(); }"] {
        assert!(!check_graphs(&[graph(owner, None)]).is_empty(), "{owner}");
    }
}
#[test]
fn commit_consults_skip_test_items_but_not_test_looking_production_files() {
    let test = "#[cfg(test)] fn probe(request: &ExecutionRequest) { request.consult(); }";
    assert!(
        check_graphs(&[graph(VALID, Some(("src/mvcc/transaction/probes.rs", test)))]).is_empty()
    );
    assert!(!check_graphs(&[graph(
        VALID,
        Some((
            "src/mvcc/transaction/tests.rs",
            "fn production(request: &ExecutionRequest) { request.consult(); }"
        ))
    )])
    .is_empty());
}
