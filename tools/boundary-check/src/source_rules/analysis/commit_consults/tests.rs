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

fn assert_count(found: Vec<crate::diagnostics::Diagnostic>, count: usize) {
    assert_eq!(found.len(), count, "{found:?}");
    for diagnostic in found {
        assert_eq!(diagnostic.code().as_str(), "BC7011_COMMIT_REQUEST_CONSULT");
    }
}

#[test]
fn commit_consults_refuse_each_other_request_liveness_spelling() {
    for code in [
        "fn bypass(execution: &ExecutionRequest) { execution.check_live(); }",
        "fn bypass(execution: &ExecutionRequest) { crate::execution::check_request(execution); }",
        "fn bypass() { let hidden = ExecutionRequest::check_live; }",
        "use worth_execution::ExecutionRequest::check_live as hidden;",
        "fn bypass(execution: &ExecutionRequest) { let alias = execution; alias.check_live(); }",
        "fn bypass(execution: &ExecutionRequest) { inspect!(execution.r#check_live()); }",
        "fn bypass(policy: &ExecutionAllocationPolicy) { let f = |policy: &ExecutionRequest| policy.check_live(); }",
        "#[cfg(test)] use ExecutionAllocationPolicy as Policy; type Policy = ExecutionRequest; fn bypass(execution: &Policy) { execution.check_live(); }",
    ] {
        assert_count(
            check_graphs(&[graph(VALID, Some(("src/mvcc/transaction/commit.rs", code)))]),
            1,
        );
    }
}

#[test]
fn commit_consults_cover_the_commit_strategy_entry() {
    assert_count(
        check_graphs(&[graph(
            VALID,
            Some((
                "src/commit_strategies/validation.rs",
                "fn bypass(execution: &ExecutionRequest) { execution.consult(); }",
            )),
        )]),
        1,
    );
}

#[test]
fn commit_consults_preserve_allocation_policy_safe_points() {
    assert_count(check_graphs(&[graph(VALID, Some((
        "src/mvcc/transaction/commit.rs",
        "fn stage(execution: &ExecutionRequest, policy: &ExecutionAllocationPolicy) { consult_stage(execution, Validation); policy.check_live(); }",
    )))]), 0);
}
#[test]
fn commit_consults_may_skip_relational_in_a_declared_mock_fixture() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/canonical_order_rules");
    assert!(root.is_dir());
    assert!(!root.join(super::CRATE).exists());
    assert_count(super::validate(&root), 0);
}

#[test]
fn commit_consults_admit_only_the_named_stage_owner() {
    assert_count(
        check_graphs(&[graph(
            VALID,
            Some((
                "src/mvcc/transaction/commit.rs",
                "fn commit(request: &ExecutionRequest) { consult_stage(request, Validation); }",
            )),
        )]),
        0,
    );
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
        assert!(found[0].message().contains("bypasses the commit stage"));
        assert_count(found, 1);
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
        assert_count(
            check_graphs(&[graph(VALID, Some(("src/mvcc/transaction/commit.rs", code)))]),
            1,
        );
    }
}
#[test]
fn commit_consults_keep_the_owner_allowance_at_one_function_and_one_consult() {
    for owner in ["fn consult_stage_control(request: &ExecutionRequest) {}",
        "fn consult_stage_control(request: &ExecutionRequest) { request.consult(); request.consult(); }",
        "fn consult_stage_control(request: &ExecutionRequest) { request.consult(); } fn bypass(request: &ExecutionRequest) { request.consult(); }"] {
        assert_count(check_graphs(&[graph(owner, None)]), 1);
    }
}
#[test]
fn commit_consults_skip_test_items_but_not_test_looking_production_files() {
    let test = "#[cfg(test)] fn probe(request: &ExecutionRequest) { request.consult(); }";
    assert_count(
        check_graphs(&[graph(VALID, Some(("src/mvcc/transaction/probes.rs", test)))]),
        0,
    );
    assert_count(
        check_graphs(&[graph(
            VALID,
            Some((
                "src/mvcc/transaction/tests.rs",
                "fn production(request: &ExecutionRequest) { request.consult(); }",
            )),
        )]),
        1,
    );
}

mod repository_roots;
