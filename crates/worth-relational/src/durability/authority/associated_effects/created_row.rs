//! Borrow original created-row fields from the SAME readmitted canonical artifact.
use super::{ReadmittedRelationalAssociatedEffect, RelationalAssociatedEffectRefusal as Refusal};
use crate::identity::data::{EntityId, KindId};
use crate::publication::patch::data::{PublishedAuthoritativeRecordPatch, RecordStructuralChange};
use crate::transactions::data::RecordRef;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};

/// Source-backed creation data, without a snapshot, runtime handle or mutation
/// authority. Construction is private to the Native readmitted-effect owner.
#[derive(Debug)]
pub struct ReadmittedRelationalCreatedEntity<'source> {
    entity: EntityId,
    kind: KindId,
    patch: &'source PublishedAuthoritativeRecordPatch,
}
impl ReadmittedRelationalCreatedEntity<'_> {
    /// Upper bound for copying this row's complete authoritative value storage.
    pub fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.patch
            .authoritative_patch
            .owned_allocation_capacity_bytes()
    }
    pub const fn entity_id(&self) -> EntityId {
        self.entity
    }
    pub const fn kind_id(&self) -> KindId {
        self.kind
    }
    /// Created Query registration rows publish their complete struct aspect.
    /// Missing, cleared or mixed field-patch grammar is never guessed into an image.
    pub fn field(&self, aspect: &AspectKey, field: &FieldKey) -> Option<&AspectValue> {
        let patch = &self.patch.authoritative_patch;
        if patch.whole_clear_for(aspect)
            || patch.field_sets_for(aspect).next().is_some()
            || patch.field_clears_for(aspect).next().is_some()
        {
            return None;
        }
        patch.struct_set_for(aspect)?.get(field)
    }
}

impl ReadmittedRelationalAssociatedEffect {
    /// Read the original creation delta, not a later mutable current row.
    /// This performs bounded admitted-artifact metadata work only; it opens
    /// no additional snapshot and cannot reconstruct a live result.
    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, Refusal> {
        read_created_entity(
            execution,
            self.semantic_result(),
            self.source_envelope(),
            entity,
        )
    }
}

pub(super) fn read_created_entity<'original>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    semantic: &crate::transactions::data::RelationalCommitSemanticResult,
    envelope: &'original crate::history::data::CanonicalCommitEnvelope,
    entity: EntityId,
) -> Result<Option<ReadmittedRelationalCreatedEntity<'original>>, Refusal> {
    crate::execution::request_stage::optional_stage(
        execution,
        |context, _memory| {
            let mut created = None;
            for value in semantic.created_entities() {
                crate::execution::request_stage::checkpoint(context, 1).map_err(|cause| {
                    Refusal::Recovery(crate::durability::data::RelationalRecoveryStop::Request(
                        cause.into(),
                    ))
                })?;
                if value.assigned == entity {
                    created = Some(value);
                }
            }
            let Some(created) = created else {
                return Ok(None);
            };
            let mut found = None;
            for patch in &envelope.patch.authoritative_record_patches {
                crate::execution::request_stage::checkpoint(context, 1).map_err(|cause| {
                    Refusal::Recovery(crate::durability::data::RelationalRecoveryStop::Request(
                        cause.into(),
                    ))
                })?;
                if patch.target != RecordRef::Entity(entity) {
                    continue;
                }
                if found.is_some() || patch.structural_change != RecordStructuralChange::Created {
                    return Err(Refusal::SubjectMismatch);
                }
                found = Some(patch);
            }
            match found {
                Some(patch) => Ok(Some(ReadmittedRelationalCreatedEntity {
                    entity,
                    kind: created.key.kind,
                    patch,
                })),
                None => Err(Refusal::SubjectMismatch),
            }
        },
        |_| 0,
    )
    .map_err(|cause| {
        Refusal::Recovery(crate::durability::data::RelationalRecoveryStop::Request(
            cause,
        ))
    })?
}
