//! Intersect a changed record's installed coverage with the selected field batch.
use super::work::MaintenanceWork;
use crate::identity::data::KindId;
use crate::indexes::data::{
    DerivedIndexId, DerivedIndexMaintenanceDenialKind as Denial, InstalledIndexRouting,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn positions(
    routing: &InstalledIndexRouting,
    pending: &BTreeMap<DerivedIndexId, usize>,
    exact: Option<&super::field_changes::FieldChanges>,
    record: crate::transactions::data::RecordRef,
    entity: bool,
    kinds: impl Iterator<Item = KindId>,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<usize>, Denial> {
    let mut selected = BTreeSet::new();
    if let Some(exact) = exact {
        if let Some(indexes) = exact.get(record, work)? {
            for index in indexes {
                work.lookup(pending.len(), 1)?;
                if let Some(position) = pending.get(index) {
                    work.ordered::<usize, ()>(selected.len(), 1, 0)?;
                    selected.insert(*position);
                }
            }
        }
        return Ok(selected);
    }
    for kind in kinds {
        work.lookup(routing.kind_count(), 1)?;
        let indexes = if entity {
            routing.entities(kind)
        } else {
            routing.relations(kind)
        };
        if pending.len() < indexes.len() {
            for (index, position) in pending {
                work.lookup(indexes.len(), 1)?;
                if indexes.binary_search(index).is_ok() {
                    work.ordered::<usize, ()>(selected.len(), 1, 0)?;
                    selected.insert(*position);
                }
            }
        } else {
            for index in indexes {
                work.lookup(pending.len(), 1)?;
                if let Some(position) = pending.get(index) {
                    work.ordered::<usize, ()>(selected.len(), 1, 0)?;
                    selected.insert(*position);
                }
            }
        }
    }
    Ok(selected)
}
