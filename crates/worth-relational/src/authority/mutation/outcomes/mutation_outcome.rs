use crate::identity::data::{EntityId, KindId, RelationId};
use worth_foundational::facade::AuthoritativeRecordAspectState;

use super::{MutationEvent, RecordMutation};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MutationOutcome {
    pub(crate) changes: Vec<RecordMutation>,
    pub(crate) events: Vec<MutationEvent>,
}

impl MutationOutcome {
    pub(crate) fn with_capacity(change_count: usize, event_count: usize) -> Self {
        Self {
            changes: Vec::with_capacity(change_count),
            events: Vec::with_capacity(event_count),
        }
    }

    pub(crate) fn entity_created(
        entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    ) -> Self {
        let mut outcome = Self::with_capacity(1, 1);
        outcome.record_change(RecordMutation::EntityCreated {
            entity_id,
            kind_id,
            authoritative_patch,
        });
        outcome.record_event(MutationEvent::EntityCreated { entity_id, kind_id });
        outcome
    }

    pub(crate) fn entity_updated_with_authoritative_patch(
        entity_id: EntityId,
        kind_id: KindId,
        old_authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
        new_authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
        authoritative_patch: worth_foundational::facade::AuthoritativeRecordAspectPatch,
    ) -> Self {
        let mut outcome = Self::with_capacity(1, 1);
        outcome.record_change(RecordMutation::EntityUpdated {
            entity_id,
            kind_id,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch: Some(authoritative_patch),
        });
        outcome.record_event(MutationEvent::EntityUpdated { entity_id });
        outcome
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
        }
    }

    pub(crate) fn entity_deleted(entity_id: EntityId) -> Self {
        let mut outcome = Self::with_capacity(0, 1);
        outcome.record_event(MutationEvent::EntityDeleted { entity_id });
        outcome
    }

    pub(crate) fn entity_replaced(
        replaced_entity_id: EntityId,
        replacement_entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    ) -> Self {
        let mut outcome = Self::with_capacity(1, 1);
        outcome.record_change(RecordMutation::EntityCreated {
            entity_id: replacement_entity_id,
            kind_id,
            authoritative_patch,
        });
        outcome.record_event(MutationEvent::EntityReplaced {
            replaced_entity_id,
            replacement_entity_id,
            kind_id,
        });
        outcome
    }

    pub(crate) fn relation_created(
        relation_id: RelationId,
        source: EntityId,
        target: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    ) -> Self {
        let mut outcome = Self::with_capacity(1, 1);
        outcome.record_change(RecordMutation::RelationCreated {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
        });
        outcome.record_event(MutationEvent::RelationCreated {
            relation_id,
            source,
            target,
            kind_id,
        });
        outcome
    }

    pub(crate) fn relation_updated(
        relation_id: RelationId,
        kind_id: KindId,
        old_source: EntityId,
        old_target: EntityId,
        new_source: EntityId,
        new_target: EntityId,
        old_authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
        new_authoritative_aspect_state: Option<AuthoritativeRecordAspectState>,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    ) -> Self {
        let mut outcome = Self::with_capacity(1, 1);
        outcome.record_change(RecordMutation::RelationUpdated {
            relation_id,
            kind_id,
            old_source,
            old_target,
            new_source,
            new_target,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch,
        });
        outcome.record_event(MutationEvent::RelationUpdated { relation_id });
        outcome
    }

    pub(crate) fn relation_deleted(relation_id: RelationId) -> Self {
        let mut outcome = Self::with_capacity(0, 1);
        outcome.record_event(MutationEvent::RelationDeleted { relation_id });
        outcome
    }

    pub(crate) fn record_change(&mut self, change: RecordMutation) {
        self.changes.push(change);
    }

    pub(crate) fn record_event(&mut self, event: MutationEvent) {
        self.events.push(event);
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.changes.extend(other.changes);
        self.events.extend(other.events);
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
