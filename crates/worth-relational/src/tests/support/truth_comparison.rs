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
        .unwrap();
    let recovered_envelope = recovered_runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
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
    assert_durable_commit_meaning_eq(execution, &original_envelope, &recovered_envelope);
}

pub(crate) fn assert_durable_commit_meaning_eq(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    expected: &crate::history::data::CanonicalCommitEnvelope,
    actual: &crate::history::data::CanonicalCommitEnvelope,
) {
    // The original artifact omits diagnostics/cache DATA (milestone-3 durable
    // projection table). Compare every other field, not a receipt-only proxy.
    assert_eq!(
        expected.recorded_original_inputs(),
        actual.recorded_original_inputs()
    );
    assert_eq!(
        durable_commit_data(execution, expected),
        durable_commit_data(execution, actual)
    );
}

pub(crate) fn durable_commit_meaning_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    expected: &crate::history::data::CanonicalCommitEnvelope,
    actual: &crate::history::data::CanonicalCommitEnvelope,
) -> bool {
    expected.recorded_original_inputs() == actual.recorded_original_inputs()
        && durable_commit_data(execution, expected) == durable_commit_data(execution, actual)
}

fn durable_commit_data(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::history::data::CanonicalCommitEnvelope,
) -> crate::history::data::CanonicalCommitEnvelope {
    let mut copied = source.copy_original_for_test(execution);
    let body = copied.body_mut_for_test();
    body.diagnostics_summary.entries.clear();
    body.derived_index_artifacts = Default::default();
    copied
}
