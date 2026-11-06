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
    entity: bool,
    kinds: impl Iterator<Item = KindId>,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<usize>, Denial> {
    let mut selected = BTreeSet::new();
    for kind in kinds {
        work.lookup(routing.kind_count(), 1)?;
        let indexes = if entity {
            routing.entities(kind)
        } else {
            routing.relations(kind)
        };
        for index in indexes {
            work.lookup(pending.len(), 1)?;
            if let Some(position) = pending.get(index) {
                work.ordered::<usize, ()>(selected.len(), 1, 0)?;
                selected.insert(*position);
            }
        }
    }
    Ok(selected)
}
