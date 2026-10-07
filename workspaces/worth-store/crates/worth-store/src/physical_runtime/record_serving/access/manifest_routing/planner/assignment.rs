use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PersistedRecordIdentity,
};

use super::super::ManifestDiscoveryCounterSnapshot;

pub(super) fn assign_updates(
    children: &[ManifestBlockReference],
    updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    drops: &BTreeSet<PersistedRecordIdentity>,
    discovery: &mut ManifestDiscoveryCounterSnapshot,
) -> Vec<(
    BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    BTreeSet<PersistedRecordIdentity>,
)> {
    let mut assigned = vec![(BTreeMap::new(), BTreeSet::new()); children.len()];
    for (record, placement) in updates {
        let (index, comparisons) =
            super::super::super::counted_search::partition_point(children, |child| {
                child.last() < *record
            });
        discovery.observe_comparisons(comparisons);
        let index = index.min(children.len().saturating_sub(1));
        assigned[index].0.insert(*record, *placement);
    }
    for record in drops {
        let (index, comparisons) =
            super::super::super::counted_search::partition_point(children, |child| {
                child.last() < *record
            });
        discovery.observe_comparisons(comparisons);
        let index = index.min(children.len().saturating_sub(1));
        assigned[index].1.insert(*record);
    }
    assigned
}
