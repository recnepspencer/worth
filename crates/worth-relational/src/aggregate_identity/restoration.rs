//! Rebind private memberships only to actual Native consequence allocations.
use crate::{
    identity::data::{EntityId, KindId, RelationId},
    publication::data::{AuthoredRetirementRefusal, PublicationError},
    storage::overlay::{CurrentPartitionAccess, WorkingState},
    transactions::data::{CommitCreatedRelationBindings, RecordRef, TransactionCommitError},
};
use worth_execution::MapKernelContext;
use worth_foundational::facade::{AspectKey, AuthoritativeRecordAspectState};

#[derive(Clone, Debug)]
pub(crate) struct RestoredMembership {
    pub(crate) original: RelationId,
    kind: KindId,
    source: EntityId,
    payload: Option<AuthoritativeRecordAspectState>,
    _funding: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl RestoredMembership {
    pub(crate) fn capture(
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
    pub(crate) fn resolve(
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
        context.checkpoint(entities.len() as u64).map_err(stop)?;
        let source = entities
            .iter()
            .find(|(old, _)| *old == self.source)
            .ok_or_else(refused)?
            .1;
        let mut selected = None;
        let aspect = AspectKey::new(super::projected_storage::MEMBERSHIP_ASPECT)
            .expect("fixed Native membership aspect");
        for (reference, id) in created.iter() {
            context.checkpoint(1).map_err(stop)?;
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
            context
                .checkpoint(
                    self.payload
                        .as_ref()
                        .map_or(0, |p| p.owned_allocation_capacity_bytes())
                        as u64
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
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
