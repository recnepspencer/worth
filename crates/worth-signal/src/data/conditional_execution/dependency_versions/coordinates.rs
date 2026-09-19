use std::cmp::Ordering;

use super::{dependency_aspects, Aspect, AspectMask, NodeId, SignalError, MAX_ASPECTS};
use crate::data::conditional_execution::execution::work;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::RetainedStoragePreparation;

type Coordinate = (NodeId, Aspect, Option<PartitionSubscription>);

pub(super) fn collect(
    node: NodeId,
    aspects: AspectMask,
    entries: &[crate::data::dependency::DependencySnapshotEntry],
    work: &mut RetainedStoragePreparation,
) -> Result<Vec<Coordinate>, SignalError> {
    // This first pass reads fixed-size lengths only, before cloning payloads.
    work::reserve(work, entries.len().checked_add(MAX_ASPECTS))?;
    let local_count = dependency_aspects(aspects).count();
    let count = work::checked(work, local_count.checked_add(entries.len()))?;
    work::checked(
        work,
        count
            .checked_mul(
                std::mem::size_of::<Coordinate>()
                    .max(std::mem::size_of::<super::SignalConditionalDependencyVersion>()),
            )
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    let mut copied_bytes = 0usize;
    let mut longest_scope = 0usize;
    for entry in entries {
        let bytes = match &entry.scope {
            Some(scope) => work::checked(
                work,
                scope
                    .partition
                    .0
                    .len()
                    .checked_add(scope.detail.as_ref().map_or(0, String::len)),
            )?,
            None => 0,
        };
        copied_bytes = work::checked(work, copied_bytes.checked_add(bytes))?;
        longest_scope = longest_scope.max(bytes);
    }
    work::reserve(
        work,
        normalization_bound(count, copied_bytes, longest_scope),
    )?;
    let mut coordinates = Vec::with_capacity(count);
    coordinates.extend(dependency_aspects(aspects).map(|aspect| (node, aspect, None)));
    coordinates.extend(
        entries
            .iter()
            .map(|entry| (entry.source, entry.aspect, entry.scope.clone())),
    );
    sort(&mut coordinates);
    coordinates.dedup();
    Ok(coordinates)
}

/// Units are coordinate visits/comparisons/moves plus scope bytes traversed.
/// A comparison charges both string operands, not only an edge count. Heap
/// construction and extraction each descend at most the tree height; a descent
/// makes at most two comparisons and one swap. Duplicate inputs remain charged.
/// This bounds normalization, not the later owners' scoped-version lookup work.
fn normalization_bound(count: usize, copied_bytes: usize, longest_scope: usize) -> Option<usize> {
    let height = if count > 1 {
        usize::BITS as usize - count.leading_zeros() as usize
    } else {
        0
    };
    let descents = count.checked_add(count / 2)?.checked_mul(height)?;
    let heap = descents.checked_mul(longest_scope.checked_mul(4)?.checked_add(4)?)?;
    let dedup = count
        .saturating_sub(1)
        .checked_mul(longest_scope.checked_mul(2)?.checked_add(1)?)?;
    heap.checked_add(dedup)?
        .checked_add(copied_bytes)?
        .checked_add(count.checked_mul(3)?)?
        .checked_add(MAX_ASPECTS)
}

// An in-place heap sort gives this owner an explicit comparison/move bound;
// no implementation-dependent sort allocation or uncounted comparator loop.
fn sort(coordinates: &mut [Coordinate]) {
    for root in (0..coordinates.len() / 2).rev() {
        sift_down(coordinates, root);
    }
    for end in (1..coordinates.len()).rev() {
        coordinates.swap(0, end);
        sift_down(&mut coordinates[..end], 0);
    }
}

fn sift_down(coordinates: &mut [Coordinate], mut root: usize) {
    while root < coordinates.len() / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < coordinates.len()
            && compare(&coordinates[child], &coordinates[child + 1]).is_lt()
        {
            child += 1;
        }
        if compare(&coordinates[root], &coordinates[child]).is_ge() {
            break;
        }
        coordinates.swap(root, child);
        root = child;
    }
}

fn compare(left: &Coordinate, right: &Coordinate) -> Ordering {
    (left.0.index(), left.0.generation(), left.1.index(), &left.2).cmp(&(
        right.0.index(),
        right.0.generation(),
        right.1.index(),
        &right.2,
    ))
}

#[cfg(test)]
mod tests;
