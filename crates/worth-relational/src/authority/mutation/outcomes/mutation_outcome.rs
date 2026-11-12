mod memory;
use super::MutationAspectPatch;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::substrate::{EntityExtra, RelationExtra, StorageAllocation};

use super::{MutationEvent, RecordMutation};

#[derive(Debug, Default)]
pub(crate) struct MutationOutcome {
    pub(crate) changes: Vec<RecordMutation>,
    pub(crate) events: Vec<MutationEvent>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl MutationOutcome {
    pub(crate) fn with_capacity(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        change_count: usize,
        event_count: usize,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::default();
        outcome.reserve(execution, change_count, event_count)?;
        Ok(outcome)
    }

    pub(crate) fn entity_created(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<MutationAspectPatch>,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 1, 1)?;
        outcome.record_change(
            execution,
            RecordMutation::EntityCreated {
                entity_id,
                kind_id,
                authoritative_patch,
            },
        )?;
        outcome.record_event(
            execution,
            MutationEvent::EntityCreated { entity_id, kind_id },
        )?;
        Ok(outcome)
    }

    pub(crate) fn entity_updated_with_authoritative_patch(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        old_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
        new_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
        authoritative_patch: MutationAspectPatch,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 1, 1)?;
        outcome.record_change(
            execution,
            RecordMutation::EntityUpdated {
                entity_id,
                kind_id,
                old_authoritative_aspect_state,
                new_authoritative_aspect_state,
                authoritative_patch: Some(authoritative_patch),
            },
        )?;
        outcome.record_event(execution, MutationEvent::EntityUpdated { entity_id })?;
        Ok(outcome)
    }

    /// A record brought back under judgement without being changed.
    ///
    /// Nothing happened to the record, so the outcome carries no change and no
    /// event: the demand's whole effect is that the slot is now touched and the
    /// rules that govern its kind will run.
    pub(crate) const fn record_revalidated() -> Self {
        Self {
            changes: Vec::new(),
            events: Vec::new(),
            _memory: None,
        }
    }

    pub(crate) fn entity_deleted(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 0, 1)?;
        outcome.record_event(execution, MutationEvent::EntityDeleted { entity_id })?;
        Ok(outcome)
    }

    pub(crate) fn entity_replaced(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        replaced_entity_id: EntityId,
        replacement_entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<MutationAspectPatch>,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 1, 1)?;
        outcome.record_change(
            execution,
            RecordMutation::EntityCreated {
                entity_id: replacement_entity_id,
                kind_id,
                authoritative_patch,
            },
        )?;
        outcome.record_event(
            execution,
            MutationEvent::EntityReplaced {
                replaced_entity_id,
                replacement_entity_id,
                kind_id,
            },
        )?;
        Ok(outcome)
    }

    pub(crate) fn relation_created(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
        source: EntityId,
        target: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<MutationAspectPatch>,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 1, 1)?;
        outcome.record_change(
            execution,
            RecordMutation::RelationCreated {
                relation_id,
                kind_id,
                source,
                target,
                authoritative_patch,
            },
        )?;
        outcome.record_event(
            execution,
            MutationEvent::RelationCreated {
                relation_id,
                source,
                target,
                kind_id,
            },
        )?;
        Ok(outcome)
    }

    pub(crate) fn relation_updated(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
        kind_id: KindId,
        old_source: EntityId,
        old_target: EntityId,
        new_source: EntityId,
        new_target: EntityId,
        old_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
        new_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
        authoritative_patch: Option<MutationAspectPatch>,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 1, 1)?;
        outcome.record_change(
            execution,
            RecordMutation::RelationUpdated {
                relation_id,
                kind_id,
                old_source,
                old_target,
                new_source,
                new_target,
                old_authoritative_aspect_state,
                new_authoritative_aspect_state,
                authoritative_patch,
            },
        )?;
        outcome.record_event(execution, MutationEvent::RelationUpdated { relation_id })?;
        Ok(outcome)
    }

    pub(crate) fn relation_deleted(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let mut outcome = Self::with_capacity(execution, 0, 1)?;
        outcome.record_event(execution, MutationEvent::RelationDeleted { relation_id })?;
        Ok(outcome)
    }

    pub(crate) fn record_change(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        change: RecordMutation,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        self.reserve(
            execution,
            self.changes
                .len()
                .checked_add(1)
                .ok_or_else(Self::overflow)?,
            self.events.len(),
        )?;
        self.changes.push(change);
        Ok(())
    }
    pub(crate) fn record_event(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        event: MutationEvent,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        self.reserve(
            execution,
            self.changes.len(),
            self.events
                .len()
                .checked_add(1)
                .ok_or_else(Self::overflow)?,
        )?;
        self.events.push(event);
        Ok(())
    }
    pub(crate) fn extend(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        other: Self,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        self.reserve(
            execution,
            self.changes
                .len()
                .checked_add(other.changes.len())
                .ok_or_else(Self::overflow)?,
            self.events
                .len()
                .checked_add(other.events.len())
                .ok_or_else(Self::overflow)?,
        )?;
        let Self {
            changes,
            events,
            _memory,
        } = other;
        self.changes.extend(changes);
        self.events.extend(events);
        drop(_memory);
        Ok(())
    }

    pub(crate) fn set_last_event_count(&mut self, count: usize) {
        if let Some(
            MutationEvent::BulkEntitiesCreated {
                count: event_count, ..
            }
            | MutationEvent::BulkRelationsCreated {
                count: event_count, ..
            },
        ) = self.events.last_mut()
        {
            *event_count = count;
        }
    }
}

impl PartialEq for MutationOutcome {
    fn eq(&self, other: &Self) -> bool {
        self.changes == other.changes && self.events == other.events
    }
}
impl Eq for MutationOutcome {}

#[cfg(test)]
mod tests;
