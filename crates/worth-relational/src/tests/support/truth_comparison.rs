use super::*;

pub(crate) fn assert_stable_aspect_truth_bundle_eq(
    expected: &AspectTruthBundle,
    actual: &AspectTruthBundle,
) {
    assert_eq!(expected.visible_truth, actual.visible_truth);
    assert_eq!(
        expected.entity_history_digests,
        actual.entity_history_digests
    );
    assert_eq!(
        expected.relation_history_digests,
        actual.relation_history_digests
    );
    assert_eq!(
        expected.lineage_history_digests,
        actual.lineage_history_digests
    );
}

pub(crate) fn assert_recovered_commit_truth_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    original_runtime: &RelationalRuntime,
    recovered_runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
    entity_ids: &[crate::facade::identity::EntityId],
    relation_ids: &[RelationId],
    lineage_ids: &[LineageId],
) {
    let original_envelope = original_runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .unwrap();
    let recovered_envelope = recovered_runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .unwrap();
    let original_bundle = capture_aspect_truth_bundle(
        execution,
        original_runtime,
        entity_ids,
        relation_ids,
        lineage_ids,
    );
    let recovered_bundle = capture_aspect_truth_bundle(
        execution,
        recovered_runtime,
        entity_ids,
        relation_ids,
        lineage_ids,
    );

    assert_stable_aspect_truth_bundle_eq(&original_bundle, &recovered_bundle);
    assert_eq!(original_envelope, recovered_envelope);
}
