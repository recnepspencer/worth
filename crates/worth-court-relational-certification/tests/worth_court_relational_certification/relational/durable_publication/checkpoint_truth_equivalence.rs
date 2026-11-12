//! Compare the original durable meaning and each named family's public truth.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::{BranchId, CanonicalCommitEnvelope, CommitId},
    identity::{EntityId, LineageId, RelationId},
    runtime::RelationalRuntime,
};

pub(super) struct TruthSubjects<'a> {
    pub entities: &'a [EntityId],
    pub relations: &'a [RelationId],
    pub lineages: &'a [LineageId],
}
pub(super) fn assert_recovered_commit_truth(
    execution: &ExecutionRequest<'_, '_>,
    original: &RelationalRuntime,
    recovered: &RelationalRuntime,
    commit_id: CommitId,
    subjects: TruthSubjects<'_>,
) {
    use super::merge_execution_observation::capture_aspect_truth_bundle;
    assert_eq!(
        capture_aspect_truth_bundle(execution, original, subjects.entities),
        capture_aspect_truth_bundle(execution, recovered, subjects.entities),
    );
    let main = BranchId("main".into());
    for relation in subjects.relations {
        assert_eq!(
            original
                .history()
                .relation_aspect_history_with_trace(execution, &main, *relation, None)
                .unwrap()
                .aspect_history_digest(),
            recovered
                .history()
                .relation_aspect_history_with_trace(execution, &main, *relation, None)
                .unwrap()
                .aspect_history_digest(),
        );
    }
    for lineage in subjects.lineages {
        assert_eq!(
            original
                .history()
                .lineage_entity_aspect_history_with_trace(execution, &main, *lineage, None)
                .unwrap()
                .lineage_aspect_resolution_digest(),
            recovered
                .history()
                .lineage_entity_aspect_history_with_trace(execution, &main, *lineage, None)
                .unwrap()
                .lineage_aspect_resolution_digest(),
        );
    }
    let before = original
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .unwrap();
    let after = recovered
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        before.original_input_observation(),
        after.original_input_observation()
    );
    assert_eq!(durable_meaning(&before), durable_meaning(&after));
}
fn durable_meaning(envelope: &CanonicalCommitEnvelope) -> serde_json::Value {
    let mut value = crate::source_wire_value::read(&rmp_serde::to_vec_named(envelope).unwrap());
    // The original durable projection omits diagnostic entries and derived caches.
    value["diagnostics_summary"]["entries"]
        .as_array_mut()
        .unwrap()
        .clear();
    value
        .as_object_mut()
        .unwrap()
        .remove("derived_index_artifacts")
        .unwrap();
    value
}
