//! Project the exact changed record identities from sealed partition slots.
use super::{ChangeWork, ChangedRecord, MaintenanceRefusal, RecordChangeState};
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::residency::RelationalRegionHeadView;
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind, SlotView};
use crate::transactions::data::primitives::RecordRef;

pub(crate) fn project_records(
    id: PartitionId,
    before: Option<RelationalRegionHeadView<'_>>,
    after: &PartitionState,
    journal: &PartitionMutationJournal,
    work: &mut ChangeWork<'_>,
) -> Result<Vec<ChangedRecord>, MaintenanceRefusal> {
    let mut records = Vec::new();
    for &slot in &journal.entity_slots {
        work.prepare(65, 256)?;
        let identity = |generation| RecordRef::Entity(EntityId::new(id, slot as u64, generation));
        records.push(ChangedRecord {
            before: before.as_ref().and_then(|old| {
                state::<EntityRecordKind>(old.entity_arena().get_slot(slot), identity)
            }),
            after: state::<EntityRecordKind>(after.entity_arena.get_slot(slot), identity),
        });
    }
    for &slot in &journal.relation_slots {
        work.prepare(65, 256)?;
        let identity =
            |generation| RecordRef::Relation(RelationId::new(id, slot as u64, generation));
        records.push(ChangedRecord {
            before: before.as_ref().and_then(|old| {
                state::<RelationRecordKind>(old.relation_arena().get_slot(slot), identity)
            }),
            after: state::<RelationRecordKind>(after.relation_arena.get_slot(slot), identity),
        });
    }
    Ok(records)
}

fn state<K: RecordKind>(
    view: Option<SlotView<'_, K>>,
    record: impl FnOnce(u32) -> RecordRef,
) -> Option<RecordChangeState> {
    let view = view?;
    Some(RecordChangeState {
        record: record(view.generation()),
        kind: view.kind_id()?,
        lifecycle: view.lifecycle(),
        retired_at: view.retired_at(),
    })
}
