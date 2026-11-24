//! The single Native entity-aspect preparation used by judgment and mutation.
use super::{field_authoring_candidate::FieldAuthoringDomain, record_aspect_patch};
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use crate::contracts::transactions::data::intents::mutation_intent::{
    ApplyEntityAspectPatchIntent, UpdateEntityFieldsIntent,
};
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use crate::contracts::transactions::data::outcomes::conflict_class::{
    ConflictClass, EntityFieldUpdateMissingState,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_foundational::facade::PortablePatchReadmissionPurpose;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

pub struct PreparedEntityAspectMutation {
    pub(crate) kind_id: crate::contracts::identity::data::primitives::KindId,
    pub(crate) authoritative_aspect_state: Option<
        worth_relational_storage::internal_port::storage::substrate::storage_allocation::StorageAllocation<worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra>,
    >,
    pub(crate) new_authoritative_aspect_state: worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra,
    pub(crate) patch: crate::authority::mutation::outcomes::MutationAspectPatch,
}

impl PreparedEntityAspectMutation {
    pub fn changes_state(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        bool,
        crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >{
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
            crate::contracts::execution::request_stage::checkpoint(context, bytes as u64).map_err(
                |cause| {
                    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
                        crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                            kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
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

pub fn prepare_fields(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    intent: &UpdateEntityFieldsIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<
    PreparedEntityAspectMutation,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
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

pub fn prepare_portable(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    intent: &ApplyEntityAspectPatchIntent,
    state: &impl CurrentPartitionAccess,
    plans: &AspectContractPlanCatalog,
) -> Result<
    PreparedEntityAspectMutation,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
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
        worth_relational_storage::internal_port::storage::substrate::storage_allocation::StorageAllocation<worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra>,
    >,
    patch: crate::authority::mutation::outcomes::MutationAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<
    PreparedEntityAspectMutation,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
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
            worth_relational_storage::internal_port::storage::substrate::storage_allocation::StorageAllocation<worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra>,
        >,
    ),
    CommitConflict,
>{
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
