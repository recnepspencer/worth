//! Rebind private memberships only to actual Native consequence allocations.
use crate::contracts::execution::request_stage::checkpoint;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal;
use crate::contracts::publication::data::publication_error::{
    PublicationError, PublicationErrorOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use worth_execution::MapKernelContext;
use worth_foundational::facade::{AspectKey, AuthoritativeRecordAspectState};
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

#[derive(Clone, Debug)]
pub struct RestoredMembership {
    pub original: RelationId,
    kind: KindId,
    source: EntityId,
    payload: Option<AuthoritativeRecordAspectState>,
    _funding: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl RestoredMembership {
    pub fn capture(
        original: RelationId,
        kind: KindId,
        source: EntityId,
        payload: Option<AuthoritativeRecordAspectState>,
        funding: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            original,
            kind,
            source,
            payload,
            _funding: funding.map(std::sync::Arc::new),
        }
    }
    pub fn resolve(
        &self,
        state: &WorkingState,
        created: &CommitCreatedRelationBindings,
        entities: &[(EntityId, EntityId)],
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(RelationId, EntityId, EntityId), TransactionCommitError> {
        let refused = || {
            TransactionCommitError::publication(PublicationError::authored_retirement(
                AuthoredRetirementRefusal::IncompleteRestoration,
                RecordRef::Relation(self.original),
            ))
        };
        checkpoint(context, entities.len() as u64).map_err(stop)?;
        let source = entities
            .iter()
            .find(|(old, _)| *old == self.source)
            .ok_or_else(refused)?
            .1;
        let mut selected = None;
        let aspect = AspectKey::new(super::projected_storage::MEMBERSHIP_ASPECT)
            .expect("fixed Native membership aspect");
        for (reference, id) in created.iter() {
            checkpoint(context, 1).map_err(stop)?;
            if reference.kind_id != self.kind {
                continue;
            }
            let Some(row) = state
                .get_current_partition(id.partition_id)
                .and_then(|partition| partition.relation_arena().get(id))
            else {
                continue;
            };
            let Some(ends) = row.extra().endpoints.as_ref() else {
                continue;
            };
            if !row.is_live() || row.kind_id() != Some(self.kind) || ends.source != source {
                continue;
            }
            let current = row.extra().authoritative_aspect_state();
            checkpoint(
                context,
                self.payload
                    .as_ref()
                    .map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64
                    + current.map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64
                    + 1,
            )
            .map_err(stop)?;
            // The canonical case roster is carrier-independent. Endpoints come
            // from the actual fresh allocations; no old aggregate ID is revived.
            if self.payload.as_ref().and_then(|p| p.get(&aspect))
                != current.and_then(|p| p.get(&aspect))
            {
                continue;
            }
            if selected.replace((*id, ends.source, ends.target)).is_some() {
                return Err(refused());
            }
        }
        selected.ok_or_else(refused)
    }
}

fn stop(
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
