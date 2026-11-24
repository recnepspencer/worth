//! Compare the roots published by the Runtime against cold truth using its retained symbols.
use crate::history::data::BranchId;
use crate::tests::support::*;

#[test]
fn incremental_content_matches_cold_rebuild_after_each_real_graph_publication() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    assert_complete(&runtime, "first entity");
    let target = create_entity(execution, &runtime, "target");
    assert_complete(&runtime, "second entity and empty adjacency");
    let relation = create_relation_outcome(execution, &runtime, source, target, "edge");
    assert_complete(&runtime, "relation creation");
    delete_relation_on_branch(
        execution,
        &runtime,
        changed_relations(&relation)[0],
        BranchId("main".to_owned()),
    );
    assert_complete(&runtime, "relation deletion");
}

fn assert_complete(runtime: &crate::runtime::RelationalRuntime, stage: &str) {
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .unwrap();
    let symbols = runtime.services.symbols.interner_snapshot();
    assert!(
        root.is_complete(
            &worth_execution::ExecutionRequest::serial(
                &crate::execution::test_policy::serial_request()
            ),
            &symbols
        ),
        "{stage}: carried commitment must match cold truth"
    );
}
