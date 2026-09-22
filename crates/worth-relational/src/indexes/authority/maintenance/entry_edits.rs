use super::work::MaintenanceWork;
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexRows,
};
use std::cmp::Ordering;

pub(super) fn edit<K: Ord + Clone, R: Clone>(
    entries: &mut DerivedIndexEntryMap<K, R>,
    key: K,
    row: R,
    insert: bool,
    compare: impl Fn(&R, &R) -> Ordering,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.charge(1)?;
    let mut rows = entries.get(&key).cloned().unwrap_or_default();
    let mut comparisons = 0;
    let found = rows.binary_search_by(|candidate| {
        comparisons += 1;
        compare(candidate, &row)
    });
    work.charge(comparisons)?;
    work.counts.seek_comparisons += comparisons;
    let position = match (insert, found) {
        (true, Err(position)) | (false, Ok(position)) => position,
        _ => return Err(Denial::PriorEntryMismatch),
    };
    reserve_paths(entries.len(), &rows, work)?;
    if insert {
        rows.insert(position, row);
    } else {
        rows.remove(position);
    }
    entries.replace(key, rows);
    work.counts.entry_edits += 1;
    Ok(())
}

fn reserve_paths<R>(
    keys: usize,
    rows: &DerivedIndexRows<R>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    // im 15.1 uses 64-way B-tree and RRB nodes. Charge up to four 64-handle
    // paths for seek, split/rebalance, and publication. A binary-height charge
    // made routine multi-row commits exhaust the budget despite path sharing.
    let height = |mut n: usize| {
        let mut levels = 1;
        while n > 64 {
            n = n.div_ceil(32);
            levels += 1;
        }
        levels
    };
    let units = 256 * (height(keys) + height(rows.len()));
    work.charge(units)?;
    work.counts.path_copy_units_reserved += units;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexes::data::DerivedIndexMaintenanceBudget;
    use std::collections::BTreeMap;

    #[test]
    fn one_edit_in_large_key_and_bucket_population_charges_search_paths() {
        let seed = (0_u64..10_000)
            .map(|key| (key, vec![key]))
            .collect::<BTreeMap<_, _>>();
        let mut entries = DerivedIndexEntryMap::from(seed);
        let retained = entries.clone();
        let mut work = MaintenanceWork::new(DerivedIndexMaintenanceBudget {
            maximum_work_units: 4_000_000,
            maximum_cold_record_slots: 0,
            maximum_derived_rows: 0,
        });
        edit(&mut entries, 5_000, 20_000, true, Ord::cmp, &mut work).unwrap();
        for key in 0_u64..1_000 {
            edit(&mut entries, key, key + 20_000, true, Ord::cmp, &mut work).unwrap();
        }
        assert_eq!(work.counts.entry_edits, 1_001);
        assert!(work.counts.work_units < 2_000_000);
        assert_eq!(entries.len(), 10_000);
        assert_eq!(retained.get(&5_000).unwrap().len(), 1);
        assert_eq!(entries.get(&5_000).unwrap().len(), 2);

        let mut large_bucket = DerivedIndexEntryMap::from(BTreeMap::from([(
            1_u64,
            (0_u64..10_000).collect::<Vec<_>>(),
        )]));
        let retained_bucket = large_bucket.clone();
        let mut work = MaintenanceWork::new(DerivedIndexMaintenanceBudget {
            maximum_work_units: 10_000,
            maximum_cold_record_slots: 0,
            maximum_derived_rows: 0,
        });
        edit(&mut large_bucket, 1, 10_000, true, Ord::cmp, &mut work).unwrap();
        assert_eq!(work.counts.entry_edits, 1);
        assert!(work.counts.work_units < 10_000);
        assert_eq!(retained_bucket.get(&1).unwrap().len(), 10_000);
        assert_eq!(large_bucket.get(&1).unwrap().len(), 10_001);
    }
}
