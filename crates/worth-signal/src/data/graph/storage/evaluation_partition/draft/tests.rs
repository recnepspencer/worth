use super::*;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::lineage::LineageArtifactId;
use crate::diagnostics::replay::ReplayCursor;
use crate::facade::SignalRuntimePolicy;
use crate::logic::transaction::SignalObservationRequest;
use crate::tests::support::{evaluate_on_demand, version_ab, ASPECT_A};

fn perform_output(
    partition: &mut SignalEvaluationPartition,
    graph: &mut SignalGraph,
    node: NodeId,
    value: u64,
) -> (u64, LineageArtifactId, u64, ReplayCursor) {
    partition
        .execute(graph, |selected| {
            let observation = selected
                .begin_observation_session(SignalObservationRequest::operation())
                .unwrap();
            evaluate_on_demand(&mut *selected, node, &mut |_, _| Ok(version_ab(value, 0))).unwrap();
            selected
                .finish_optional_invalidation_execution_observation(&observation)
                .unwrap();
            drop(observation);
            assert_eq!(
                selected
                    .node_version_for_scope(node, ASPECT_A, None)
                    .unwrap(),
                value
            );
            (
                selected.cause_sets.output_commit_ordinal_for_test(),
                selected
                    .node_lineage_artifact_id(node)
                    .unwrap()
                    .expect("performed output has lineage"),
                selected
                    .diagnostics_state()
                    .lineage_records()
                    .back()
                    .expect("performed lineage record")
                    .sequence,
                selected
                    .diagnostics_state()
                    .latest_replay_cursor()
                    .expect("performed replay event"),
            )
        })
        .unwrap()
}

#[test]
fn rejected_output_and_retry_keep_distinct_issued_identities() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().build();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let draft = ConditionalEvaluationDraft::begin(
        &mut partition,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000),
    )
    .unwrap();
    let rejected_ids = perform_output(draft.partition, &mut graph, node, 7);
    // Deliberate kernel-level rejection after real output publication. This
    // tests issuance custody, not source admission or a sealed service journey.
    let rejected = draft.reject();
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_version_for_scope(node, ASPECT_A, None)
                    .unwrap(),
                0
            );
            assert!(selected.node_lineage_artifact_id(node).unwrap().is_none());
            assert!(selected.diagnostics_state().lineage_records().is_empty());
            assert!(selected.diagnostics_state().replay_events().is_empty());
            assert_eq!(
                selected.cause_sets.output_commit_ordinal_for_test(),
                rejected_ids.0
            );
        })
        .unwrap();
    let draft = ConditionalEvaluationDraft::begin(
        &mut partition,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000),
    )
    .unwrap();
    let retry_ids = perform_output(draft.partition, &mut graph, node, 9);
    draft.install();
    assert!(
        retry_ids.0 > rejected_ids.0,
        "publication ordinal is never reissued"
    );
    assert!(
        retry_ids.1 > rejected_ids.1,
        "lineage artifact identity is never reissued"
    );
    assert!(
        retry_ids.2 > rejected_ids.2,
        "lineage event sequence is never reissued"
    );
    assert!(
        retry_ids.3 > rejected_ids.3,
        "replay cursor is never reissued"
    );
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_version_for_scope(node, ASPECT_A, None)
                    .unwrap(),
                9
            );
        })
        .unwrap();
    assert!(!rejected.diagnostics_for_test()["lineage_records"]
        .as_array()
        .unwrap()
        .is_empty());
}
