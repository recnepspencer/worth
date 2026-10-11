//! Live node credit for one persistent map and its edited buckets.
use super::work::MaintenanceWork;
use crate::indexes::data::{DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexRows};
use std::{collections::BTreeMap, mem::size_of, sync::Arc};

#[derive(Default)]
pub(super) struct MapAllocation {
    nodes: usize,
    prepaid: bool,
}
impl MapAllocation {
    pub(super) fn prepaid() -> Self {
        Self {
            nodes: 0,
            prepaid: true,
        }
    }
    pub(super) fn prepare_key<K>(&self, work: &mut MaintenanceWork) -> Result<(), Denial> {
        work.prepare(
            0,
            if self.prepaid {
                0
            } else {
                (size_of::<K>() + 2 * size_of::<usize>()) as u64
            },
        )
    }

    pub(super) fn prepare<K: Ord + Clone, R: Clone>(
        &mut self,
        keys: usize,
        width: usize,
        work: &mut MaintenanceWork,
    ) -> Result<(), Denial> {
        let (path, node) = persistent_map_path_bytes::<K, R>(keys)?;
        // im 15 B-tree nodes have at least 31 keys except the root. All live
        // copied nodes fit that population plus one mutation path; reuse this
        // credit rather than charging the same uniquely owned path again.
        let live = keys
            .div_ceil(31)
            .checked_add(2)
            .and_then(|n| n.checked_add(path))
            .ok_or(Denial::WorkBudgetExceeded)?;
        let next = self
            .nodes
            .checked_add(path)
            .ok_or(Denial::WorkBudgetExceeded)?
            .min(live)
            .max(self.nodes);
        let bytes = if self.prepaid {
            0
        } else {
            (next - self.nodes)
                .checked_mul(node)
                .ok_or(Denial::WorkBudgetExceeded)?
        };
        work.lookup(keys, width)?;
        work.prepare((path * 64) as u64, bytes as u64)?;
        self.nodes = next;
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct RowAllocation {
    nodes: usize,
    prepaid: bool,
}
impl RowAllocation {
    pub(super) fn prepaid() -> Self {
        Self {
            nodes: 0,
            prepaid: true,
        }
    }
    pub(super) fn prepare<R>(
        &mut self,
        rows: &DerivedIndexRows<R>,
        insert: bool,
        work: &mut MaintenanceWork,
    ) -> Result<(), Denial> {
        let length = rows
            .len()
            .checked_add(usize::from(insert))
            .ok_or(Denial::WorkBudgetExceeded)?;
        let inline = sized_chunks::InlineArray::<Arc<R>, im::vector::Rrb<Arc<R>>>::CAPACITY;
        let path = if rows.is_inline() && length <= inline {
            0
        } else {
            5 * path_height(length) + 8
        };
        // RRB merge packs 64-slot chunks; only the four fringes can remain
        // under half full. Branches and leaves together fit twice ceil(n/32),
        // four fringe paths and one transient split/append path. A branch's
        // 64 four-word Entries, size table and both Arc/chunk headers fit
        // 332 words. Inline rows allocate only an inserted Arc<R>.
        let live = 2 * length.div_ceil(32) + 4 * path_height(length) + 4 + path;
        let next = self
            .nodes
            .checked_add(path)
            .ok_or(Denial::WorkBudgetExceeded)?
            .min(live)
            .max(self.nodes);
        let bytes = (next - self.nodes)
            .checked_mul(size_of::<usize>() * 332)
            .and_then(|n| {
                n.checked_add(if insert {
                    size_of::<R>() + 2 * size_of::<usize>()
                } else {
                    0
                })
            })
            .ok_or(Denial::WorkBudgetExceeded)?;
        work.prepare(
            (path * 64) as u64,
            if self.prepaid { 0 } else { bytes as u64 },
        )?;
        self.nodes = next;
        Ok(())
    }
}

pub(super) struct UngroupedAllocation<K> {
    pub(super) map: MapAllocation,
    rows: BTreeMap<K, RowAllocation>,
}
impl<K: Ord + Copy> UngroupedAllocation<K> {
    pub(super) fn new() -> Self {
        Self {
            map: MapAllocation::default(),
            rows: BTreeMap::new(),
        }
    }
    pub(super) fn row<R>(
        &mut self,
        key: K,
        rows: &DerivedIndexRows<R>,
        insert: bool,
        work: &mut MaintenanceWork,
    ) -> Result<(), Denial> {
        work.lookup(self.rows.len(), 1)?;
        if !self.rows.contains_key(&key) {
            work.ordered::<K, RowAllocation>(self.rows.len(), 1, 0)?;
        }
        self.rows
            .entry(key)
            .or_default()
            .prepare(rows, insert, work)
    }
}

pub(super) fn prepare_empty_map<K: Ord + Clone, R: Clone>(
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let (_, node) = persistent_map_path_bytes::<K, R>(0)?;
    work.lookup(0, 1)?;
    work.prepare(5 * 64, node as u64)
}

fn persistent_map_path_bytes<K: Ord + Clone, R: Clone>(
    keys: usize,
) -> Result<(usize, usize), Denial> {
    // im 15 copies a path, sibling and split/rebalance node at each level,
    // plus two roots: 3h+2. Each node has 64 Arc-key/vector pairs, 65 children,
    // four offsets and two Arc words. Key headers are paid per replacement.
    let node = size_of::<(Arc<K>, DerivedIndexRows<R>)>()
        .checked_mul(64)
        .and_then(|n| n.checked_add(size_of::<usize>() * 71))
        .ok_or(Denial::WorkBudgetExceeded)?;
    Ok((3 * path_height(keys) + 2, node))
}
// im uses 64-slot chunks. A non-root B-tree level is at least half full,
// so ceil(n/32) removes one worst-case level until the 64-slot root fits.
fn path_height(mut size: usize) -> usize {
    let mut levels = 1;
    while size > 64 {
        size = size.div_ceil(32);
        levels += 1;
    }
    levels
}

// Fresh maps have no shared path to copy. Reserve their complete output once,
// before the first edit; the already-funded empty root supplies one node.
pub(super) fn prepare_cold_grouped<K: Ord + Clone, R: Clone>(
    edits: &BTreeMap<K, Vec<super::entry_edits::PendingEdit<R>>>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.prepare(edits.len() as u64, 0)?;
    let keys = edits.len();
    let (path, node) = persistent_map_path_bytes::<K, R>(keys)?;
    let nodes = keys.div_ceil(31) + 2 + path;
    let mut bytes = (nodes - 1)
        .checked_mul(node)
        .ok_or(Denial::WorkBudgetExceeded)?;
    for operations in edits.values() {
        let rows = operations.len();
        let inline = sized_chunks::InlineArray::<Arc<R>, im::vector::Rrb<Arc<R>>>::CAPACITY;
        let row_nodes = if rows <= inline {
            0
        } else {
            let height = path_height(rows);
            2 * rows.div_ceil(32) + 4 * height + 4 + 5 * height + 8
        };
        let addition = row_nodes
            .checked_mul(size_of::<usize>() * 332)
            .and_then(|n| {
                rows.checked_mul(size_of::<R>() + 2 * size_of::<usize>())
                    .and_then(|payload| n.checked_add(payload))
            })
            .and_then(|n| n.checked_add(size_of::<K>() + 2 * size_of::<usize>()))
            .ok_or(Denial::WorkBudgetExceeded)?;
        bytes = bytes
            .checked_add(addition)
            .ok_or(Denial::WorkBudgetExceeded)?;
    }
    work.prepare(0, bytes as u64)
}
