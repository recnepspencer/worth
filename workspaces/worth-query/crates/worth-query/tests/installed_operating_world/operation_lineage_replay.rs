use crate::workflow_request::workflow_request;
use worth_query::facade::runtime::ExecutionRequest;
use worth_query::facade::{certification, domain};

use super::installed_operation_fixture::{lineage_workflow_workspace, LineageEvidenceScenario};
use super::operation_lineage::{bind, execute, intent};

#[test]
fn certification_replay_reexecutes_the_same_lineage_semantics() {
    let mut workspace = lineage_workflow_workspace(
        "lineage-replay",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    let original = execute(&mut workspace);
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
    )
    .unwrap();

    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_ne!(original.identity(), replay.replay_trace_identity());
    let original_lineage_width = replay
        .original_semantics()
        .stages()
        .iter()
        .flat_map(|stage| stage.lineage())
        .count();
    let replay_lineage_width = replay
        .replay_semantics()
        .stages()
        .iter()
        .flat_map(|stage| stage.lineage())
        .count();
    assert!(original_lineage_width > 0);
    assert_eq!(original_lineage_width, replay_lineage_width);
    assert!(original.lineage_report().is_some());
}
