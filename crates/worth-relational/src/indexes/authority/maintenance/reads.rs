//! Borrow the reached kind posting and read its native endpoint metadata.
use super::{
    record_metadata::{self, RelationMetadata},
    root_view::IndexRootView,
    work::MaintenanceWork,
};
use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::storage::partition::AdjacencyDirection;
pub(super) fn adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &IndexRootView,
    entity: EntityId,
    kind: KindId,
    outgoing: bool,
    work: &mut MaintenanceWork,
) -> Result<Vec<RelationMetadata>, Denial> {
    let Some(head) = view.head(execution, entity.partition_id, work)? else {
        return Ok(Vec::new());
    };
    let table = head.adjacency(if outgoing {
        AdjacencyDirection::Outgoing
    } else {
        AdjacencyDirection::Incoming
    });
    work.prepare(table.navigation_work_bound() as u64, 0)?;
    let Some(posting) = table.get(entity.slot_index()) else {
        return Ok(Vec::new());
    };
    work.prepare(posting.navigation_work_bound() as u64, 0)?;
    let mut ids = posting
        .kind_ids(kind)
        .try_iter(|units, bytes| work.prepare(units, bytes))?;
    let mut records = Vec::new();
    while let Some(id) = ids.try_next(|units, bytes| work.prepare(units, bytes))? {
        work.charge(1)?;
        work.note_adjacency_work_units(1);
        if let Some(record) = record_metadata::relation(execution, Some(view), *id, work)?
            && record.kind == kind
            && (if outgoing {
                record.source
            } else {
                record.target
            }) == entity
        {
            work.grow_vec(&mut records)?;
            records.push(record);
        }
    }
    Ok(records)
}
