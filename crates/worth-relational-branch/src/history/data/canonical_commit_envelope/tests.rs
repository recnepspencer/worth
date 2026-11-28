use crate::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraphOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResultParts;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::assemble_commit_semantic_result;
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
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::commit_log::{
    CommitAspectSummary, CommitChangeSummary, CommitHistorySummary,
};
use crate::transactions::data::intents::mutation_intent::EntityMutationIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::intents::mutation_intent::UpdateEntityFieldsIntent;
use crate::transactions::data::mutation_planning::CommitTopology;
use crate::transactions::data::outcomes::commit_semantic_result::{
    RelationalCommitPublicationMeaning,
    RelationalCommitStructuralMeaning,
};
use crate::transactions::data::outcomes::commit_summaries::CommitSchemaSummary;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::RecordRef;
use crate::transactions::data::primitives::TransactionId;
use std::sync::Arc;

pub fn envelope_with_patch_and_update_for_test(
    patch_target: RecordRef,
    update_target: EntityId,
) -> CanonicalCommitEnvelope {
    let receipt = RelationalCommitReceipt {
        commit_id: CommitId(1),
        version_id: VersionId(1),
        branch_id: BranchId("main".to_owned()),
        parents: Vec::new(),
    };
    envelope_with_patch_at(patch_target, update_target, receipt)
}

pub(crate) fn envelope_with_patch_at(
    patch_target: RecordRef,
    update_target: EntityId,
    receipt: RelationalCommitReceipt,
) -> CanonicalCommitEnvelope {
    let descriptor_semantics_version = DescriptorSemanticsVersion(1);
    let draft = crate::history::data::CanonicalCommitEnvelopeDraft::new(
        Default::default(),
        crate::history::data::CanonicalIndexDefinitionDelta::prepared(1, 1, Vec::new(), None),
        receipt.clone(),
        BranchId("main".to_string()),
        CanonicalCommitAuthorityKind::VersionedTransaction,
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
                target: patch_target.clone(),
                structural_change: RecordStructuralChange::Updated,
                relation_birth: None,
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
        descriptor_semantics_version,
    );
    // Seal directly at the canonical owner. These descriptor predicates
    // require one admitted wire value, with no claim of performed publication.
    let parent_count = receipt.parents.len();
    let semantic = assemble_commit_semantic_result(RelationalCommitSemanticResultParts {
        receipt,
        created_entities: Vec::new(),
        created_relations: Vec::new(),
        changed_records: vec![patch_target],
        structural: RelationalCommitStructuralMeaning {
            invariant_groups: Default::default(),
            commit_topology: CommitTopology::FlatEntityBatch,
            touched_partitions: vec![PartitionId::main()],
            descriptor_semantics_version,
        },
        schema: CommitSchemaSummary {
            transition: None,
            descriptor_semantics_version,
        },
        history: CommitHistorySummary {
            target_branch: "main".to_owned(),
            requested_merge_parent_count: 0,
            effective_merge_parent_count: 0,
            parent_count,
            merge_base_count: 0,
            had_previous_branch_head: parent_count != 0,
        },
        change: CommitChangeSummary {
            changed_record_count: 1,
            adjacency_delta_count: 0,
        },
        aspects: CommitAspectSummary {
            changed_entity_aspect_count: 0,
            changed_relation_aspect_count: 0,
            touched_aspects: Vec::new(),
            opaque_aspect_delta_count: 0,
            zero_aspect_structural_delta_count: 1,
        },
        publication: RelationalCommitPublicationMeaning {
            patch_record_count: 1,
            lineage_event_count: 0,
            merge_parent_count: parent_count.saturating_sub(1),
        },
        validation: Vec::new(),
        decoded_memory: None,
    });
    draft.seal(Arc::new(semantic)).unwrap()
}

#[test]
fn envelope_touched_record_refs_include_patch_and_existing_record_intents() {
    let patch_entity = RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1));
    let updated_entity = EntityId::new(PartitionId::main(), 2, 1);
    let envelope = envelope_with_patch_and_update_for_test(patch_entity.clone(), updated_entity);

    let touched = envelope.touched_record_refs();
    assert!(touched.contains(&patch_entity));
    assert!(touched.contains(&RecordRef::Entity(updated_entity)));
}

#[test]
fn committed_record_changes_for_target_filters_to_matching_record() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let envelope = envelope_with_patch_and_update_for_test(RecordRef::Entity(entity), entity);
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
    let mut envelope = envelope_with_patch_and_update_for_test(RecordRef::Entity(entity), entity);
    envelope.body_mut_for_test().descriptive_touches =
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
