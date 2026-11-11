//! The single Native entity-aspect preparation used by judgment and mutation.
use super::{field_authoring_candidate::FieldAuthoringDomain, record_aspect_patch};
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CommitConflict, ConflictClass, EntityFieldUpdateMissingState,
    RecordAspectPatchTarget, UpdateEntityFieldsIntent,
};
use worth_foundational::facade::{
    AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState, PortablePatchReadmissionPurpose,
};

pub(crate) struct PreparedEntityAspectMutation {
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
    pub(crate) new_authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
    pub(crate) patch: AuthoritativeRecordAspectPatch,
}

impl PreparedEntityAspectMutation {
    pub(crate) fn changes_state(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<bool, crate::transactions::data::TransactionCommitError> {
        // The patch owner can change only these aspects. Untouched persistent
        // map entries never become whole-record comparison inputs.
        let keys = self
            .patch
            .whole_aspect_sets()
            .map(|(key, _)| key)
            .chain(self.patch.whole_aspect_clears())
            .chain(self.patch.field_patches().map(|(key, _)| key));
        for key in keys {
            let before = self
                .authoritative_aspect_state
                .as_ref()
                .and_then(|s| s.get(key));
            let after = self
                .new_authoritative_aspect_state
                .as_ref()
                .and_then(|s| s.get(key));
            let bytes = before
                .map_or(0, |v| v.semantic_byte_width())
                .saturating_add(after.map_or(0, |v| v.semantic_byte_width()))
                .saturating_add(key.as_str().len())
                .saturating_add(2);
            crate::execution::request_stage::checkpoint(context, bytes as u64).map_err(
                |cause| {
                    crate::transactions::data::TransactionCommitError::execution(
                        crate::transactions::data::CommitExecutionDenial {
                            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                                cause.into(),
                            ),
                            partition_identity: None,
                        },
                    )
                },
            )?;
            if before != after {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub(crate) fn prepare_fields(
    intent: &UpdateEntityFieldsIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<PreparedEntityAspectMutation, CommitConflict> {
    let (kind_id, current) = current(intent.entity_id, state)?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    finish(kind_id, current, patch, target)
}

pub(crate) fn prepare_portable(
    intent: &ApplyEntityAspectPatchIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<PreparedEntityAspectMutation, CommitConflict> {
    let (kind_id, current) = current(intent.entity_id, state)?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit(
        intent.aspect_patch.clone(),
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind_id),
        target,
    )?;
    finish(kind_id, current, patch, target)
}

fn finish(
    kind_id: KindId,
    authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
    patch: AuthoritativeRecordAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<PreparedEntityAspectMutation, CommitConflict> {
    let new_authoritative_aspect_state =
        record_aspect_patch::apply(authoritative_aspect_state.as_ref(), &patch, target)?;
    Ok(PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state,
        new_authoritative_aspect_state,
        patch,
    })
}

fn current(
    entity_id: EntityId,
    state: &impl CurrentPartitionAccess,
) -> Result<(KindId, Option<AuthoritativeRecordAspectState>), CommitConflict> {
    let slot = entity_id.slot_index();
    {
        ensure_entity_target_is_current(state, entity_id)?;
        let partition = state
            .get_current_partition(entity_id.partition_id)
            .ok_or_else(|| {
                CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                    entity_id,
                    missing: EntityFieldUpdateMissingState::Partition,
                })
            })?;
        let slot_view = partition.entity_arena().get_slot(slot).ok_or_else(|| {
            CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                entity_id,
                missing: EntityFieldUpdateMissingState::Slot,
            })
        })?;
        let kind_id = slot_view.kind_id().ok_or_else(|| {
            CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                entity_id,
                missing: EntityFieldUpdateMissingState::KindId,
            })
        })?;
        Ok::<_, CommitConflict>((
            kind_id,
            slot_view.extra().authoritative_aspect_state.clone(),
        ))
    }
}
