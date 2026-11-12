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
use worth_foundational::facade::PortablePatchReadmissionPurpose;

pub(crate) struct PreparedEntityAspectMutation {
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) authoritative_aspect_state: Option<
        crate::storage::substrate::StorageAllocation<crate::storage::substrate::EntityExtra>,
    >,
    pub(crate) new_authoritative_aspect_state: crate::storage::substrate::EntityExtra,
    pub(crate) patch: crate::authority::mutation::outcomes::MutationAspectPatch,
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
            .as_patch()
            .whole_aspect_sets()
            .map(|(key, _)| key)
            .chain(self.patch.as_patch().whole_aspect_clears())
            .chain(self.patch.as_patch().field_patches().map(|(key, _)| key));
        for key in keys {
            let before = self
                .authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state())
                .and_then(|s| s.get(key));
            let after = self
                .new_authoritative_aspect_state
                .authoritative_aspect_state()
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    intent: &UpdateEntityFieldsIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<PreparedEntityAspectMutation, crate::transactions::data::TransactionCommitError> {
    let (kind_id, current) = current(intent.entity_id, state)?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        execution,
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    finish(execution, kind_id, current, patch, target)
}

pub(crate) fn prepare_portable(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    intent: &ApplyEntityAspectPatchIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<PreparedEntityAspectMutation, crate::transactions::data::TransactionCommitError> {
    let (kind_id, current) = current(intent.entity_id, state)?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit_original_portable(
        execution,
        &intent.aspect_patch,
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind_id),
        target,
    )?;
    finish(execution, kind_id, current, patch, target)
}

fn finish(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    kind_id: KindId,
    authoritative_aspect_state: Option<
        crate::storage::substrate::StorageAllocation<crate::storage::substrate::EntityExtra>,
    >,
    patch: crate::authority::mutation::outcomes::MutationAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<PreparedEntityAspectMutation, crate::transactions::data::TransactionCommitError> {
    let new_authoritative_aspect_state = record_aspect_patch::apply_entity(
        execution,
        authoritative_aspect_state
            .as_ref()
            .and_then(|extra| extra.authoritative_aspect_state()),
        &patch,
        target,
    )?;
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
) -> Result<
    (
        KindId,
        Option<
            crate::storage::substrate::StorageAllocation<crate::storage::substrate::EntityExtra>,
        >,
    ),
    CommitConflict,
> {
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
        Ok::<_, CommitConflict>((kind_id, Some(slot_view.retained_extra())))
    }
}
