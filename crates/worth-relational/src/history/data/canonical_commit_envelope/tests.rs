use super::{
    CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, RelationalDescriptiveTouchGraph,
};
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticArtifact,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchDetail, PatchOrdering, PatchPublicationMode,
    PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use crate::schema::data::{DescriptorSemanticsVersion, RelationalSchemaRegistry, SchemaVersionId};
use crate::transactions::data::{
    AspectFieldPatch, EntityMutationIntent, MergedCommitPlan, MutationIntent, RecordRef,
    TransactionId, UpdateEntityFieldsIntent,
};

fn envelope_with_patch_and_update(
    patch_target: RecordRef,
    update_target: EntityId,
) -> CanonicalCommitEnvelope {
    CanonicalCommitEnvelope::new(
        RelationalCommitReceipt {
            commit_id: CommitId(1),
            version_id: VersionId(1),
            branch_id: BranchId("main".to_string()),
            parents: vec![],
        },
        BranchId("main".to_string()),
        CanonicalCommitAuthorityKind::VersionedTransaction,
        None,
        None,
        vec![],
        vec![],
        SchemaVersionId(1),
        RelationalSchemaRegistry::new().authority_snapshot(),
        MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: update_target,
                    fields: AspectFieldPatch::default(),
                },
            ))],
        },
        CanonicalAuthoritativePatch {
            ordering: PatchOrdering::CanonicalCommitOrder,
            publication_mode: PatchPublicationMode::CommitNative,
            authoritative_record_patches: vec![PublishedAuthoritativeRecordPatch {
                target: patch_target,
                structural_change: RecordStructuralChange::Updated,
                authoritative_patch:
                    crate::publication::patch::data::PublishedAuthoritativePatch::empty(),
                semantic_changes: Vec::new(),
                contains_opaque_aspect: false,
                detail: PatchDetail::DenseBitset(vec![1]),
            }],
        },
        RelationalDescriptiveTouchGraph::exact(vec![]),
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Replay,
            DiagnosticsArtifactKind::MinimalSummary,
            DeterminismExpectation::Required,
            vec![],
        ),
        LineageFinalizationArtifact::new(
            BranchId("main".to_string()),
            FinalizedLineageEventBatch::new(vec![]),
            LineageDecisionLog::new(vec![]),
        )
        .publish(),
        DerivedIndexArtifacts::default(),
        None,
        None,
        None,
        DescriptorSemanticsVersion(1),
    )
}

#[test]
fn envelope_touched_record_refs_include_patch_and_existing_record_intents() {
    let patch_entity = RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1));
    let updated_entity = EntityId::new(PartitionId::main(), 2, 1);
    let envelope = envelope_with_patch_and_update(patch_entity.clone(), updated_entity);

    let touched = envelope.touched_record_refs();
    assert!(touched.contains(&patch_entity));
    assert!(touched.contains(&RecordRef::Entity(updated_entity)));
}

#[test]
fn committed_record_changes_for_target_filters_to_matching_record() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let envelope = envelope_with_patch_and_update(RecordRef::Entity(entity), entity);
    let target = RecordRef::Entity(entity);

    let matched = envelope
        .committed_record_changes_for_target(&target)
        .collect::<Vec<_>>();

    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].commit.commit_id, CommitId(1));
    assert_eq!(matched[0].record.target, target);
}

#[test]
fn named_envelope_wire_preserves_exact_touch_graph_and_defaults_old_missing_graph_unavailable() {
    use super::RelationalDescriptiveTouch;
    use crate::identity::data::KindId;

    let entity = EntityId::new(PartitionId::main(), 8, 1);
    let mut envelope = envelope_with_patch_and_update(RecordRef::Entity(entity), entity);
    envelope.descriptive_touches =
        RelationalDescriptiveTouchGraph::exact(vec![RelationalDescriptiveTouch::EntityLifecycle {
            entity,
            kind: KindId(1),
        }]);
    let encoded = rmp_serde::to_vec_named(&envelope).expect("named canonical envelope encodes");
    let restored: CanonicalCommitEnvelope =
        rmp_serde::from_slice(&encoded).expect("named canonical envelope decodes");
    assert_eq!(
        restored.descriptive_touches(),
        envelope.descriptive_touches()
    );

    let mut legacy = serde_json::to_value(&envelope).expect("old named map source");
    legacy
        .as_object_mut()
        .expect("named envelope map")
        .remove("descriptive_touches");
    let old: CanonicalCommitEnvelope =
        serde_json::from_value(legacy).expect("prior named envelope remains readable");
    assert_eq!(old.descriptive_touches().exact_touches(), None);
}
