//! A selected whole record is a real mutation, including unchanged field payloads.
use super::ValidatedMutationTouch as Touch;
use crate::{
    identity::data::KindId, publication::patch::data::RecordStructuralChange as Change,
    transactions::data::RecordRef,
};
pub(super) fn whole_record(record: &RecordRef, kind: KindId, change: Change) -> Touch {
    match (record, change) {
        (RecordRef::Entity(_), Change::Deleted) => Touch::DeleteEntity { kind },
        (RecordRef::Entity(_), _) => Touch::UnrepresentableEntityMutation { kind },
        (RecordRef::Relation(_), Change::Deleted) => Touch::UnlinkRelation { kind },
        (RecordRef::Relation(_), Change::Rematerialized) => Touch::LinkRelation { kind },
        (RecordRef::Relation(_), _) => Touch::UnrepresentableRelationMutation { kind },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::{EntityId, PartitionId, RelationId};
    #[test]
    fn selected_record_lifecycle_never_becomes_an_empty_or_authored_create_effect() {
        // Local transition-table inputs; these descriptive IDs grant no admission.
        let entity = RecordRef::Entity(EntityId::new(PartitionId::main(), 0, 1));
        let relation = RecordRef::Relation(RelationId::new(PartitionId::main(), 0, 1));
        let kind = KindId(7);
        assert_eq!(
            whole_record(&entity, kind, Change::Updated),
            Touch::UnrepresentableEntityMutation { kind }
        );
        assert_eq!(
            whole_record(&entity, kind, Change::Rematerialized),
            Touch::UnrepresentableEntityMutation { kind }
        );
        assert_eq!(
            whole_record(&entity, kind, Change::Deleted),
            Touch::DeleteEntity { kind }
        );
        assert_eq!(
            whole_record(&relation, kind, Change::Updated),
            Touch::UnrepresentableRelationMutation { kind }
        );
        assert_eq!(
            whole_record(&relation, kind, Change::Rematerialized),
            Touch::LinkRelation { kind }
        );
        assert_eq!(
            whole_record(&relation, kind, Change::Deleted),
            Touch::UnlinkRelation { kind }
        );
    }
}
