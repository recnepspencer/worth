//! Canonical snapshot normalization without allocated comparison keys.
mod admission;
use super::{DependencySnapshot, DependencySnapshotEntry};
use std::sync::Arc;

pub(super) fn normalize(
    snapshot: &mut DependencySnapshot,
    work: &mut crate::logic::evaluation::EvaluationWork<'_>,
) -> Result<(), crate::data::error::SignalError> {
    let comparison = admission::comparison_bound(snapshot.entries(), work)?;
    // Immutable canonical snapshots retain their backing through repeated store
    // admission. In particular, do not enter Arc::make_mut just to re-sort them.
    if snapshot
        .entries()
        .windows(2)
        .all(|pair| pair[0].compare_key(&pair[1]).is_lt())
    {
        return Ok(());
    }
    admission::mutation_bound(snapshot.entries(), comparison, work)?;
    let entries = Arc::make_mut(&mut snapshot.entries);
    let count = entries.len();
    for root in (0..count / 2).rev() {
        sift_down(entries, root);
    }
    for end in (1..count).rev() {
        entries.swap(0, end);
        sift_down(&mut entries[..end], 0);
    }
    entries.dedup_by(|next, previous| {
        if !next.compare_key(previous).is_eq() {
            return false;
        }
        // Entries are sorted by key and then version. Keep the greatest
        // version by moving it into the surviving slot, without copying scopes.
        std::mem::swap(next, previous);
        true
    });
    Ok(())
}

fn compare(left: &DependencySnapshotEntry, right: &DependencySnapshotEntry) -> std::cmp::Ordering {
    left.compare_key(right)
        .then(left.cached_version.cmp(&right.cached_version))
}

fn sift_down(entries: &mut [DependencySnapshotEntry], mut root: usize) {
    while root < entries.len() / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < entries.len() && compare(&entries[child], &entries[child + 1]).is_lt() {
            child += 1;
        }
        if !compare(&entries[root], &entries[child]).is_lt() {
            break;
        }
        entries.swap(root, child);
        root = child;
    }
}

#[cfg(test)]
mod tests;
