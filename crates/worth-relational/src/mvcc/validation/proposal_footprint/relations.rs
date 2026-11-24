//! Relation lifecycle material comes from the validated actor's real applied draft.
use super::member_restoration::{
    project_restored_member, RecordedMemberRestoration, RecordedMemberRestorationRefusal,
};
use super::relation_mutations::{
    stop, ValidatedRelationMutation, ValidatedRelationMutationFootprint,
};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::identity::data::KindId;
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;

impl ValidatedRelationalProposal {
    /// Optional relation projection. The field-only footprint does no additional work.
    ///
    /// `members` are the recorded members an inverse must restore under owners
    /// that stay live, and `member_kinds` the entity kinds its declared member
    /// families own: every birth of such a kind must be one recorded member.
    /// The member pass runs only when either is non-empty. It costs one
    /// correspondence probe per birth across the recorded members and their
    /// retained fields, then one owner-edge probe per recorded member; it
    /// never reads a roster or a thread.
    pub fn relation_mutation_footprint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        self.project_relation_mutations(execution, kinds, members, member_kinds, false)
    }
    /// The actual root writer's inverse material, excluding every later stage
    /// and producer. The range is Native-issued, never a caller-supplied subset.
    /// An uncoupled proposal is its whole root; a genuine empty root stays empty.
    pub fn authored_prefix_relation_mutation_footprint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        self.project_relation_mutations(execution, kinds, members, member_kinds, true)
    }
    fn project_relation_mutations(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
        prefix_only: bool,
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        let prefix_records = if prefix_only {
            match self
                .state
                .required_output_publication
                .as_ref()
                .map(|a| &a.publication)
            {
                Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => Some(
                    group
                        .authored_records
                        .get(..group.authored_prefix_records)
                        .ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled authored root range unavailable")
                        })?,
                ),
                _ => None,
            }
        } else {
            None
        };
        let fragments = &self
            .state
            .proposed_mutation
            .outcome
            .effect
            .publication
            .patch_fragments;
        let restoration = self
            .state
            .required_output_publication
            .as_ref()
            .and_then(|admission| admission.publication.authored_restoration());
        let corrective_root = if prefix_only {
            self.state
                .required_output_publication
                .as_ref()
                .and_then(|admission| match &admission.publication {
                    crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => group
                        .binding
                        .owners
                        .corrective_root
                        .as_ref()
                        .map(|(_, targets)| targets.elements()),
                    _ => None,
                })
        } else {
            None
        };
        let anchors = corrective_root.map_or_else(
            || {
                restoration.map_or(0, |restoration| {
                    restoration
                        .retirement
                        .authored_source_retirement()
                        .expect("Native restoration")
                        .co_anchors
                        .len()
                        + 1
                })
            },
            |targets| targets.len(),
        );
        // Both routes enter the same allocation resolution and root-membership
        // projection. The binder already refused conflict with a real plan.
        let anchor_targets =
            corrective_root.or_else(|| restoration.map(|plan| plan.entities.as_slice()));
        let restored_capacity = anchors.checked_add(members.len()).ok_or_else(|| {
            stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
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
                stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })?;
        let memory = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(|denial| {
                crate::execution::denial::allocation_commit_error(
                    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                        crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                            crate::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied {
                                denial,
                            },
                        ),
                    ),
                )
            })?;
        let mut relations = Vec::new();
        relations.try_reserve_exact(fragments.len()).map_err(|_| {
            stop(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        })?;
        let mut restored_entities = Vec::new();
        restored_entities
            .try_reserve_exact(restored_capacity)
            .map_err(|_| {
                stop(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
            })?;
        let mut member_refusal = None;
        execution.run_native_with_checkpoint(|context| {
            if let Some(targets) = anchor_targets {
                for (original, target) in targets.iter().take(anchors) {
                    let created = &self.state.proposed_mutation.outcome.created_entities;
                    checkpoint(context, created.resolve_work_bound(target).ok_or_else(|| stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(stop)?;
                    let actual = created.resolve(target).ok_or_else(|| crate::transactions::data::outcomes::transaction_commit_error::origin_denial("authored inverse anchor has no actual allocation"))?;
                    if let Some(root) = prefix_records {
                        checkpoint(context, root.len() as u64).map_err(stop)?;
                        if !root.contains(&RecordRef::Entity(actual)) {
                            return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial("restoration anchor is outside the actual authored root"));
                        }
                    }
                    checkpoint(context, restored_entities.len() as u64).map_err(stop)?;
                    if restored_entities.iter().any(|(known, allocated)| known == original || *allocated == actual) {
                        return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial("ambiguous actual authored root correspondence"));
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
                    && fragment.structural_change == crate::publication::patch::data::RecordStructuralChange::Created {
                    let partition = self.state.proposed_mutation.state.get_current_partition(entity.partition_id);
                    if let Some(kind) = partition.as_ref().and_then(|partition| partition.entity_arena().get(&entity)).filter(|row| row.is_live()).and_then(|row| row.kind_id()) {
                        let state = crate::validation::engine::state_view::InvariantStateView::new(
                            &self.state.proposed_mutation.state,
                            self.state.evidence.proposed_version(),
                        );
                        project_restored_member(&state, entity, kind, members, member_kinds, &mut restored_entities, &mut member_refusal, context)?;
                    }
                }
                let RecordRef::Relation(relation) = fragment.target else { continue; };
                if let Some(admission) = &self.state.required_output_publication
                    && let crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } = &admission.publication
                {
                    checkpoint(context, group.authored_records.len() as u64).map_err(stop)?;
                    if !group.authored_records.contains(&fragment.target) { continue; }
                }
                let before_partition = self.state.prepared.selected_branch_state.get_current_partition(relation.partition_id);
                let before_row = before_partition.as_ref().and_then(|partition| partition.relation_arena().get(&relation));
                let kind = before_row.as_ref().and_then(|row| row.kind_id()).or_else(|| fragment.relation_birth.as_ref().map(|birth| birth.kind));
                checkpoint(context, kinds.len() as u64).map_err(stop)?;
                let Some(kind) = kind.filter(|kind| kinds.contains(kind)) else { continue; };
                let before = before_row.filter(|row| row.is_live()).and_then(|row| row.extra().endpoints.as_ref().map(|ends| (ends.source, ends.target)));
                let after_partition = self.state.proposed_mutation.state.get_current_partition(relation.partition_id);
                let after = after_partition.as_ref().and_then(|partition| partition.relation_arena().get(&relation))
                    .filter(|row| row.is_live()).and_then(|row| row.extra().endpoints.as_ref().map(|ends| (ends.source, ends.target)));
                let after = if matches!(fragment.structural_change, crate::publication::patch::data::RecordStructuralChange::Deleted | crate::publication::patch::data::RecordStructuralChange::RetainedForAudit) { None } else { after };
                if before == after { continue; }
                // Several fragments can describe one record. Retain its original and final truth once.
                checkpoint(context, relations.len() as u64).map_err(stop)?;
                if !relations.iter().any(|old: &ValidatedRelationMutation| old.relation == relation) {
                    let allocated_endpoint = match (before, after) {
                        (None, Some((source, target))) => {
                            let created = &self.state.proposed_mutation.outcome.created_entities;
                            created.creation_reference(source, || checkpoint(context, 1).map_err(stop))?.is_some()
                                || created.creation_reference(target, || checkpoint(context, 1).map_err(stop))?.is_some()
                        }
                        _ => false,
                    };
                    relations.push(ValidatedRelationMutation { relation, kind, audit_retained: matches!(fragment.structural_change, crate::publication::patch::data::RecordStructuralChange::RetainedForAudit), before, after, allocated_endpoint });
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
}
