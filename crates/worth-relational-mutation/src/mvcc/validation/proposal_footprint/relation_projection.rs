//! Project funded relation changes and member correspondence from selected states.
use super::member_restoration::{
    project_restored_member, RecordedMemberRestoration, RecordedMemberRestorationRefusal,
};
use super::relation_mutations::{
    stop, ValidatedRelationMutation, ValidatedRelationMutationFootprint,
};
use crate::authority::mutation::canonical_deltas::FoundationalPatchFragment;
use crate::contracts::execution::request_stage::checkpoint;
use crate::contracts::identity::data::primitives::{KindId, VersionId};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::primitives::{CreatedEntityRef, RecordRef};
use worth_execution::ExecutionRequest;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub struct RelationProjectionInputs<'a> {
    pub before: &'a dyn CurrentPartitionAccess,
    pub after: &'a WorkingState,
    pub proposed_version: VersionId,
    pub fragments: &'a [FoundationalPatchFragment],
    pub created_entities: &'a CommitCreatedEntityBindings,
    pub prefix_records: Option<&'a [RecordRef]>,
    pub authored_records: Option<&'a [RecordRef]>,
    pub anchors: usize,
    pub anchor_targets: Option<&'a [(EntityId, CreatedEntityRef)]>,
}

pub fn project_relations(
    execution: &ExecutionRequest<'_, '_>,
    inputs: RelationProjectionInputs<'_>,
    kinds: &[KindId],
    members: &[RecordedMemberRestoration<'_>],
    member_kinds: &[KindId],
) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
    let RelationProjectionInputs {
        anchors,
        anchor_targets,
        fragments,
        prefix_records,
        ..
    } = inputs;
    let restored_capacity = anchors.checked_add(members.len()).ok_or_else(|| {
        stop(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
    })?;
    let bytes = fragments
        .len()
        .checked_mul(size_of::<ValidatedRelationMutation>())
        .and_then(|bytes| {
            restored_capacity
                .checked_mul(size_of::<(EntityId, EntityId)>())
                .and_then(|extra| bytes.checked_add(extra))
        })
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            stop(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    let memory = execution
        .allocation_policy()
        .reserve_memory(bytes)
        .map_err(|denial| {
            crate::contracts::execution::denial::allocation_commit_error(
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                    crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                        crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied {
                            denial,
                        },
                    ),
                ),
            )
        })?;
    let mut relations = Vec::new();
    relations
        .try_reserve_exact(fragments.len())
        .map_err(|_| stop(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed))?;
    let mut restored_entities = Vec::new();
    restored_entities
        .try_reserve_exact(restored_capacity)
        .map_err(|_| stop(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed))?;
    let mut member_refusal = None;
    execution.run_native_with_checkpoint(|context| {
        if let Some(targets) = anchor_targets {
            for (original, target) in targets.iter().take(anchors) {
                let created = inputs.created_entities;
                checkpoint(context, created.resolve_work_bound(target).ok_or_else(|| stop(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(stop)?;
                let actual = created.resolve(target).ok_or_else(|| crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("authored inverse anchor has no actual allocation"))?;
                if let Some(root) = prefix_records {
                    checkpoint(context, root.len() as u64).map_err(stop)?;
                    if !root.contains(&RecordRef::Entity(actual)) {
                        return Err(crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("restoration anchor is outside the actual authored root"));
                    }
                }
                checkpoint(context, restored_entities.len() as u64).map_err(stop)?;
                if restored_entities.iter().any(|(known, allocated)| known == original || *allocated == actual) {
                    return Err(crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("ambiguous actual authored root correspondence"));
                }
                restored_entities.push((*original, actual));
            }
        }
        for fragment in fragments {
            checkpoint(context, 1).map_err(stop)?;
            if let Some(root) = prefix_records {
                checkpoint(context, root.len() as u64).map_err(stop)?;
                if !root.contains(&fragment.target) { continue; }
            }
            if !(members.is_empty() && member_kinds.is_empty())
                && let RecordRef::Entity(entity) = fragment.target
                && fragment.structural_change == crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Created {
                let partition = inputs.after.get_current_partition(entity.partition_id);
                if let Some(kind) = partition.as_ref().and_then(|partition| partition.entity_arena().get(&entity)).filter(|row| row.is_live()).and_then(|row| row.kind_id()) {
                    let state = crate::validation::engine::state_view::InvariantStateView::new(
                        inputs.after,
                        inputs.proposed_version,
                    );
                    project_restored_member(&state, entity, kind, members, member_kinds, &mut restored_entities, &mut member_refusal, context)?;
                }
            }
            let RecordRef::Relation(relation) = fragment.target else { continue; };
            if let Some(authored) = inputs.authored_records {
                checkpoint(context, authored.len() as u64).map_err(stop)?;
                if !authored.contains(&fragment.target) { continue; }
            }
            let before_partition = inputs.before.get_current_partition(relation.partition_id);
            let before_row = before_partition.as_ref().and_then(|partition| partition.relation_arena().get(&relation));
            let kind = before_row.as_ref().and_then(|row| row.kind_id()).or_else(|| fragment.relation_birth.as_ref().map(|birth| birth.kind));
            checkpoint(context, kinds.len() as u64).map_err(stop)?;
            let Some(kind) = kind.filter(|kind| kinds.contains(kind)) else { continue; };
            let before = before_row.filter(|row| row.is_live()).and_then(|row| row.extra().endpoints.as_ref().map(|ends| (ends.source, ends.target)));
            let after_partition = inputs.after.get_current_partition(relation.partition_id);
            let after = after_partition.as_ref().and_then(|partition| partition.relation_arena().get(&relation))
                .filter(|row| row.is_live()).and_then(|row| row.extra().endpoints.as_ref().map(|ends| (ends.source, ends.target)));
            let after = if matches!(fragment.structural_change, crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Deleted | crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange::RetainedForAudit) { None } else { after };
            if before == after { continue; }
            // Several fragments can describe one record. Retain its original and final truth once.
            checkpoint(context, relations.len() as u64).map_err(stop)?;
            if !relations.iter().any(|old: &ValidatedRelationMutation| old.relation == relation) {
                let allocated_endpoint = match (before, after) {
                    (None, Some((source, target))) => {
                        let created = inputs.created_entities;
                        created.creation_reference(source, || checkpoint(context, 1).map_err(stop))?.is_some()
                            || created.creation_reference(target, || checkpoint(context, 1).map_err(stop))?.is_some()
                    }
                    _ => false,
                };
                relations.push(ValidatedRelationMutation { relation, kind, audit_retained: matches!(fragment.structural_change, crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange::RetainedForAudit), before, after, allocated_endpoint });
            }
        }
        for member in members {
            checkpoint(context, restored_entities.len() as u64).map_err(stop)?;
            let Some((_, fresh)) = restored_entities.iter().find(|(old, _)| *old == member.original) else {
                member_refusal = Some(RecordedMemberRestorationRefusal::Incomplete(member.original));
                break;
            };
            checkpoint(context, relations.len() as u64).map_err(stop)?;
            if !relations.iter().any(|edge| edge.kind() == member.owner.0 && edge.after() == Some((*fresh, member.owner.1))) && member_refusal.is_none() {
                member_refusal = Some(RecordedMemberRestorationRefusal::Foreign(member.original));
            }
        }
        Ok::<(), TransactionCommitError>(())
    }).map_err(stop)?.0?;
    Ok(ValidatedRelationMutationFootprint {
        relations,
        restored_entities: match member_refusal {
            Some(refusal) => Err(refusal),
            None => Ok(restored_entities),
        },
        _memory: memory,
    })
}
