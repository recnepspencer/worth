//! Relation lifecycle material comes from the validated actor's real applied draft.
use super::member_restoration::{RecordedMemberRestoration, RecordedMemberRestorationRefusal};
use super::ValidatedRelationalProposal;
use crate::execution::request_stage::checkpoint;
use crate::{
    identity::data::{EntityId, KindId, RelationId},
    storage::overlay::CurrentPartitionAccess,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedRelationMutation {
    relation: RelationId,
    kind: KindId,
    audit_retained: bool,
    before: Option<(EntityId, EntityId)>,
    after: Option<(EntityId, EntityId)>,
    allocated_endpoint: bool,
}
impl ValidatedRelationMutation {
    pub const fn relation(&self) -> RelationId {
        self.relation
    }
    pub const fn kind(&self) -> KindId {
        self.kind
    }
    pub const fn is_audit_retained(&self) -> bool {
        self.audit_retained
    }
    pub const fn before(&self) -> Option<(EntityId, EntityId)> {
        self.before
    }
    pub const fn after(&self) -> Option<(EntityId, EntityId)> {
        self.after
    }
    /// A born relation with an endpoint this same draft allocated. Its prior
    /// absence is structural: no earlier relation can name an entity that did
    /// not exist, so no read is needed to know it had no prior endpoints.
    pub const fn has_allocated_endpoint(&self) -> bool {
        self.allocated_endpoint
    }
}
/// Privately constructed from one invariant-approved draft; selectors grant no effects.
#[derive(Debug)]
pub struct ValidatedRelationMutationFootprint {
    relations: Vec<ValidatedRelationMutation>,
    restored_entities: Result<Vec<(EntityId, EntityId)>, RecordedMemberRestorationRefusal>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ValidatedRelationMutationFootprint {
    pub fn restored_entities(
        &self,
    ) -> Result<&[(EntityId, EntityId)], RecordedMemberRestorationRefusal> {
        self.restored_entities
            .as_deref()
            .map_err(|refusal| *refusal)
    }
    pub fn relations(&self) -> &[ValidatedRelationMutation] {
        &self.relations
    }
}
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
        let anchors = restoration.map_or(0, |restoration| {
            restoration
                .retirement
                .authored_source_retirement()
                .expect("Native restoration")
                .co_anchors
                .len()
                + 1
        });
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
                    crate::transactions::data::TransactionCommitError::conflict(
                        crate::transactions::data::CommitConflict::new(
                            crate::transactions::data::ConflictClass::ExecutionAllocationDenied {
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
            if let Some(restoration) = restoration {
                for (original, target) in restoration.entities.iter().take(anchors) {
                    let created = &self.state.proposed_mutation.outcome.created_entities;
                    checkpoint(context, created.resolve_work_bound(target).ok_or_else(|| stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(stop)?;
                    let actual = created.resolve(target).ok_or_else(|| crate::mvcc::origin_denial("authored inverse anchor has no actual allocation"))?;
                    restored_entities.push((*original, actual));
                }
            }
            for fragment in fragments {
                checkpoint(context, 1).map_err(stop)?;
                if !(members.is_empty() && member_kinds.is_empty())
                    && let RecordRef::Entity(entity) = fragment.target
                    && fragment.structural_change == crate::publication::patch::data::RecordStructuralChange::Created {
                    let partition = self.state.proposed_mutation.state.get_current_partition(entity.partition_id);
                    if let Some(kind) = partition.as_ref().and_then(|partition| partition.entity_arena().get(&entity)).filter(|row| row.is_live()).and_then(|row| row.kind_id()) {
                        self.project_restored_member(entity, kind, members, member_kinds, &mut restored_entities, &mut member_refusal, context)?;
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
pub(super) fn stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
