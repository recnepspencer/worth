use super::work::MaintenanceWork;
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexRows,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub(super) struct PendingEdit<R> {
    pub(super) row: R,
    pub(super) insert: bool,
}

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

pub(super) fn grouped<K: Ord + Clone, R: Clone>(
    entries: &mut DerivedIndexEntryMap<K, R>,
    edits: BTreeMap<K, Vec<PendingEdit<R>>>,
    compare: impl Fn(&R, &R) -> Ordering,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for (key, operations) in edits {
        let mut rows = entries.get(&key).cloned().unwrap_or_default();
        let mut map_reserved = false;
        for operation in operations {
            let mut comparisons = 0;
            let found = rows.binary_search_by(|candidate| {
                comparisons += 1;
                compare(candidate, &operation.row)
            });
            work.charge(comparisons)?;
            work.counts.seek_comparisons += comparisons;
            let position = match (operation.insert, found) {
                (true, Err(position)) | (false, Ok(position)) => position,
                _ => return Err(Denial::PriorEntryMismatch),
            };
            if !map_reserved {
                reserve_map_path(entries.len(), work)?;
                map_reserved = true;
            }
            reserve_row_path(rows.len(), work)?;
            if operation.insert {
                rows.insert(position, operation.row);
            } else {
                rows.remove(position);
            }
            work.counts.entry_edits += 1;
        }
        if map_reserved {
            entries.replace(key, rows);
        }
    }
    Ok(())
}

fn reserve_paths<R>(
    keys: usize,
    rows: &DerivedIndexRows<R>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let units = path_reservation(keys) + path_reservation(rows.len());
    work.charge(units)?;
    work.counts.path_copy_units_reserved += units;
    Ok(())
}

fn reserve_map_path(keys: usize, work: &mut MaintenanceWork) -> Result<(), Denial> {
    reserve_handle_path(keys, work)
}

fn reserve_row_path(rows: usize, work: &mut MaintenanceWork) -> Result<(), Denial> {
    reserve_handle_path(rows, work)
}

fn reserve_handle_path(size: usize, work: &mut MaintenanceWork) -> Result<(), Denial> {
    let units = path_reservation(size);
    work.charge(units)?;
    work.counts.path_copy_units_reserved += units;
    Ok(())
}

fn path_reservation(size: usize) -> usize {
    // An empty map/vector has no retained path to copy. Otherwise reserve one
    // maximum-size im node per changed level. The new generation owns its
    // mutable path after the first copy; charging split/rebalance copies on
    // every edit overcounts shared batches by the size of the House scene.
    // Payloads are shared Arcs and are not copied by the persistent update.
    if size == 0 {
        1
    } else {
        64 * path_height(size)
    }
}

fn path_height(mut size: usize) -> usize {
    let mut levels = 1;
    while size > 64 {
        size = size.div_ceil(32);
        levels += 1;
    }
    levels
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

    #[test]
    fn grouped_edits_replace_one_shared_key_path_and_preserve_prior_rows() {
        let mut entries = DerivedIndexEntryMap::from(BTreeMap::from([(1_u64, vec![1_u64, 3])]));
        let retained = entries.clone();
        let mut sequential = entries.clone();
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: 10_000,
            maximum_cold_record_slots: 0,
            maximum_derived_rows: 0,
        };
        let mut grouped_work = MaintenanceWork::new(budget);
        let mut sequential_work = MaintenanceWork::new(budget);
        let operations = [(2, true), (4, true), (1, false)];
        let mut edits = BTreeMap::new();
        for (row, insert) in operations {
            grouped_work.charge(1).unwrap();
            edits
                .entry(1)
                .or_insert_with(Vec::new)
                .push(PendingEdit { row, insert });
            edit(
                &mut sequential,
                1,
                row,
                insert,
                Ord::cmp,
                &mut sequential_work,
            )
            .unwrap();
        }
        grouped(&mut entries, edits, Ord::cmp, &mut grouped_work).unwrap();
        assert_eq!(entries, sequential);
        assert_eq!(
            retained
                .get(&1)
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert_eq!(grouped_work.counts.entry_edits, 3);
        assert_eq!(
            grouped_work.counts.seek_comparisons,
            sequential_work.counts.seek_comparisons
        );
        assert!(
            grouped_work.counts.path_copy_units_reserved
                < sequential_work.counts.path_copy_units_reserved
        );
    }
}
