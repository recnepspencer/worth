//! Preparation correspondence borrows the immutable original intent keys.
use super::*;
use crate::identity::data::KindId;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreatedEntityRef, CreatedRelationRef, EntityReference, TransactionCommitError,
};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct CreatedEntityKey<'a> {
    partition: crate::identity::data::PartitionId,
    kind: KindId,
    client: &'a ClientKey,
}
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct CreatedRelationKey<'a> {
    partition: crate::identity::data::PartitionId,
    kind: KindId,
    client: &'a ClientKey,
    source: &'a EntityReference,
    target: &'a EntityReference,
}
impl CreatedEntityKey<'_> {
    fn owned(self) -> CreatedEntityRef {
        CreatedEntityRef {
            partition_id: self.partition,
            kind_id: self.kind,
            client_key: self.client.clone(),
        }
    }
}
impl CreatedRelationKey<'_> {
    fn owned(self) -> CreatedRelationRef {
        CreatedRelationRef {
            partition_id: self.partition,
            kind_id: self.kind,
            client_key: self.client.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
        }
    }
}
impl<'a> MutationWorkspace<'a> {
    /// Existing live public-result construction remains an ordinary owned copy.
    /// The Complete fold never calls this conversion.
    pub(crate) fn into_created_bindings(
        self,
    ) -> (
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
    ) {
        let entity = self
            .created_entities
            .iter()
            .map(|(key, id)| (key.owned(), *id))
            .collect();
        let relation = self
            .created_relations
            .iter()
            .map(|(key, id)| (key.owned(), *id))
            .collect();
        (
            crate::transactions::data::CommitCreatedEntityBindings::from_owner_map(
                entity,
                self.symbols.lookup(),
            ),
            crate::transactions::data::CommitCreatedRelationBindings::from_owner_map(
                relation,
                self.symbols.lookup(),
            ),
        )
    }
    pub(crate) fn register_created_entity(
        &mut self,
        partition: crate::identity::data::PartitionId,
        kind: KindId,
        client: &'a ClientKey,
        id: EntityId,
    ) -> Result<(), TransactionCommitError> {
        self.created_entities
            .insert_cloned_native_for_request(
                &CreatedEntityKey {
                    partition,
                    kind,
                    client,
                },
                &id,
                self.execution,
            )
            .map_err(map_stop)
    }
    pub(crate) fn register_created_relation(
        &mut self,
        partition: crate::identity::data::PartitionId,
        kind: KindId,
        client: &'a ClientKey,
        source: &'a EntityReference,
        target: &'a EntityReference,
        id: RelationId,
    ) -> Result<(), TransactionCommitError> {
        self.created_relations
            .insert_cloned_native_for_request(
                &CreatedRelationKey {
                    partition,
                    kind,
                    client,
                    source,
                    target,
                },
                &id,
                self.execution,
            )
            .map_err(map_stop)
    }
    pub(crate) fn resolve_entity_reference(&self, reference: &EntityReference) -> Option<EntityId> {
        match reference {
            EntityReference::Existing(id) => Some(*id),
            EntityReference::Created(created) => self
                .created_entities
                .get(&CreatedEntityKey {
                    partition: created.partition_id,
                    kind: created.kind_id,
                    client: &created.client_key,
                })
                .copied(),
        }
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> TransactionCommitError {
    use worth_execution::ExecutionOrderedMapDenial as Denial;
    let cause = match cause {
        Denial::Request(cause) => cause.into(),
        Denial::Allocation(_) => crate::execution::RelationalExecutionDenialCause::AllocationFailed,
        Denial::CapacityOverflow => {
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow
        }
    };
    TransactionCommitError::execution(cause.into())
}
