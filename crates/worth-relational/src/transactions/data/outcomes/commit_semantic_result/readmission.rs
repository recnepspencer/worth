//! Check recorded original facts against the enclosing canonical publication.
use super::{
    RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCreatedEndpoint,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::primitives::RecordRef;

mod creations;

pub(crate) fn validate_envelope(
    semantic: &RelationalCommitSemanticResult,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Refusal> {
    if semantic.receipt != envelope.commit
        || semantic.history.target_branch != semantic.receipt.branch_id.0
    {
        return Err(Refusal::ReceiptMismatch);
    }
    let same_transition = match (&semantic.schema.transition, &envelope.schema_transition) {
        (None, None) => true,
        (Some(summary), Some(artifact)) => summary.matches_artifact(artifact),
        _ => false,
    };
    if !same_transition
        || semantic.schema.descriptor_semantics_version != envelope.descriptor_semantics_version
        || semantic.structural.descriptor_semantics_version != envelope.descriptor_semantics_version
    {
        return Err(Refusal::SchemaMismatch);
    }
    if semantic
        .changed_records
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
        || envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                semantic
                    .changed_records
                    .binary_search(&patch.target)
                    .is_err()
            })
        || semantic.change.changed_record_count != semantic.changed_records.len()
    {
        return Err(Refusal::ChangedRecordsMismatch);
    }
    if semantic.publication.patch_record_count != envelope.patch.authoritative_record_patches.len()
        || semantic.publication.lineage_event_count != envelope.lineage_events().len()
        || semantic.publication.merge_parent_count
            != semantic.receipt.parents.len().saturating_sub(1)
        || semantic.history.parent_count != semantic.receipt.parents.len()
        || semantic.history.merge_base_count != envelope.merge_base_commits.len()
        || semantic
            .structural
            .touched_partitions
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || semantic
            .aspects
            .touched_aspects
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(Refusal::PublicationMismatch);
    }
    validate_creations(semantic, envelope)?;
    // An original committed observation may name the predecessor version.
    // Its version is an execution fact, not the newly published version.
    if semantic
        .validation
        .iter()
        .any(|execution| !execution.is_consistent())
    {
        return Err(Refusal::ValidationMismatch);
    }
    Ok(())
}

fn validate_creations(
    semantic: &RelationalCommitSemanticResult,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Refusal> {
    if semantic
        .created_entities
        .windows(2)
        .any(|pair| pair[0].key >= pair[1].key)
        || semantic
            .created_relations
            .windows(2)
            .any(|pair| pair[0].key >= pair[1].key)
    {
        return Err(Refusal::NonCanonicalCorrespondence);
    }
    let patches = &envelope.patch.authoritative_record_patches;
    // A typed ordinal roster preserves FIRST matching Created patch even
    // for invalid duplicate targets; no private std node layout is assumed.
    let mut created_patches = Vec::with_capacity(patches.len());
    created_patches.extend(patches.iter().enumerate().filter_map(|(index, patch)| {
        (patch.structural_change == RecordStructuralChange::Created).then_some(index)
    }));
    created_patches.sort_unstable_by(|left, right| {
        patches[*left]
            .target
            .cmp(&patches[*right].target)
            .then(left.cmp(right))
    });
    let created_patch = |target: &RecordRef| {
        let slot = created_patches.partition_point(|index| &patches[*index].target < target);
        created_patches
            .get(slot)
            .map(|index| &patches[*index])
            .filter(|patch| &patch.target == target)
    };
    let count = semantic
        .created_entities
        .len()
        .checked_add(semantic.created_relations.len())
        .ok_or(Refusal::CreateCardinalityOverflow)?;
    let mut records = Vec::with_capacity(count);
    for created in &semantic.created_entities {
        if created.key.partition != created.assigned.partition_id {
            return Err(Refusal::CreatePartitionMismatch);
        }
        records.push(RecordRef::Entity(created.assigned));
    }
    let resolve = |reference: &RelationalCreatedEndpoint| match reference {
        RelationalCreatedEndpoint::Existing(id) => Some(*id),
        RelationalCreatedEndpoint::Created(key) => semantic
            .created_entities
            .binary_search_by(|created| created.key.cmp(key))
            .ok()
            .map(|index| semantic.created_entities[index].assigned),
    };
    for created in &semantic.created_relations {
        if created.key.partition != created.assigned.partition_id {
            return Err(Refusal::CreatePartitionMismatch);
        }
        let source = resolve(&created.key.source).ok_or(Refusal::CreatedEndpointMissing)?;
        let target = resolve(&created.key.target).ok_or(Refusal::CreatedEndpointMissing)?;
        records.push(RecordRef::Relation(created.assigned));
        let birth = created_patch(&RecordRef::Relation(created.assigned))
            .ok_or(Refusal::CreatedPatchMissing {
                record: crate::transactions::data::primitives::ExistingRecordTarget::Relation(
                    created.assigned,
                ),
            })?
            .relation_birth;
        if birth.is_none_or(|birth| {
            birth.kind != created.key.kind || birth.source != source || birth.target != target
        }) {
            return Err(Refusal::RelationBirthMismatch {
                assigned: created.assigned,
            });
        }
    }
    records.sort_unstable();
    if let Some(pair) = records.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(Refusal::DuplicateAssignedCreation {
            record: record_target(&pair[0]),
        });
    }
    if let Some(record) = records
        .iter()
        .find(|record| created_patch(record).is_none())
    {
        return Err(Refusal::CreatedPatchMissing {
            record: record_target(record),
        });
    }
    creations::validate_original_keys(semantic, envelope)?;
    Ok(())
}

fn record_target(
    record: &RecordRef,
) -> crate::transactions::data::primitives::ExistingRecordTarget {
    use crate::transactions::data::primitives::ExistingRecordTarget;
    match record {
        RecordRef::Entity(id) => ExistingRecordTarget::Entity(*id),
        RecordRef::Relation(id) => ExistingRecordTarget::Relation(*id),
    }
}
