use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{CurrentPhysicalRecordPlacement, PersistedRecordIdentity};

use super::super::ManifestLookupFailure;

pub(super) struct LeafMerge {
    pub(super) entries: Vec<CurrentPhysicalRecordPlacement>,
    pub(super) inserted: u64,
    pub(super) removed: u64,
}

/// This is the incremental path-copy leaf transformation, before the writer
/// emits authenticated successor blocks. The caller checks the global drop
/// count because a missing identity may sort into another leaf.
pub(super) fn merge_leaf(
    entries: Vec<CurrentPhysicalRecordPlacement>,
    updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    drops: &BTreeSet<PersistedRecordIdentity>,
) -> LeafMerge {
    let mut merged = entries
        .into_iter()
        .map(|entry| (entry.record(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut inserted = 0;
    let mut removed = 0;
    for (record, placement) in updates {
        if merged.insert(*record, *placement).is_none() {
            inserted += 1;
        }
    }
    for record in drops {
        if merged.remove(record).is_some() {
            removed += 1;
        }
    }
    LeafMerge {
        entries: merged.into_values().collect(),
        inserted,
        removed,
    }
}

pub(super) fn require_complete_drop(
    removed: u64,
    requested: usize,
) -> Result<(), ManifestLookupFailure> {
    (usize::try_from(removed).ok() == Some(requested))
        .then_some(())
        .ok_or(ManifestLookupFailure::Damaged)
}

#[cfg(test)]
mod tests;
