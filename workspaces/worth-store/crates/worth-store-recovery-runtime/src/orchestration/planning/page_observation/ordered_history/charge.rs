//! What one root step charges against the manifest-entry limit.
//!
//! A step is charged for what its member declares it wrote, before any of
//! the step's reads, so the charge is the same on every run of one workload:
//! it does not depend on which blocks the records landed in.

use std::cmp::Ordering;

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, FreeSpaceKey,
    PersistedPhysicalRecoveryOperation, RecordFreeSpaceManifestEntry,
};
use worth_store_recovery_physics::AdmittedRootStepMemberView;

/// One for the step's result root, and for a WAL member its placements,
/// segment updates, inline allocations and the records its directory drops.
/// A retirement edge has no member; its free-space difference is charged once
/// both sides are read.
pub(super) fn step(member: Option<&AdmittedRootStepMemberView<'_>>) -> Option<usize> {
    let Some(member) = member else {
        return Some(1);
    };
    let projection = member.materialization();
    let dropped = match projection.operation() {
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } => retirement.dropped_records().len(),
        _ => 0,
    };
    declared(
        projection.placements().len(),
        projection.segment_updates().len(),
        projection.root_state().inline_allocations().len(),
        dropped,
    )
}

fn declared(
    placements: usize,
    segment_updates: usize,
    inline_allocations: usize,
    dropped: usize,
) -> Option<usize> {
    1_usize
        .checked_add(placements)?
        .checked_add(segment_updates)?
        .checked_add(inline_allocations)?
        .checked_add(dropped)
}

/// The free-space entries one side of a retirement edge holds and the other
/// does not. Both sides are in canonical key order.
pub(super) fn net_free_difference(
    source: &[RecordFreeSpaceManifestEntry],
    result: &[RecordFreeSpaceManifestEntry],
) -> Option<usize> {
    let (mut left, mut right, mut different) = (0, 0, 0_usize);
    while left < source.len() && right < result.len() {
        match FreeSpaceKey::from(source[left]).cmp(&FreeSpaceKey::from(result[right])) {
            Ordering::Less => {
                left += 1;
                different = different.checked_add(1)?;
            }
            Ordering::Greater => {
                right += 1;
                different = different.checked_add(1)?;
            }
            Ordering::Equal => {
                if source[left] != result[right] {
                    different = different.checked_add(2)?;
                }
                left += 1;
                right += 1;
            }
        }
    }
    different
        .checked_add(source.len() - left)?
        .checked_add(result.len() - right)
}

/// What the headers of one root say about the trees under it: the root
/// header counts its records and the free-space header its entries. Nothing
/// in a header counts segment pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TreeHeaders {
    root_node_capacity: u16,
    free_node_capacity: u16,
    routes: u64,
    free_entries: u64,
}

impl TreeHeaders {
    pub(super) const fn of(
        root: &DurablePhysicalRootManifest,
        free_space: &DurableFreeSpaceManifestHeader,
    ) -> Self {
        Self {
            root_node_capacity: root.node_capacity(),
            free_node_capacity: free_space.node_capacity(),
            routes: root.record_count(),
            free_entries: free_space.entry_count(),
        }
    }
}

/// A step that changes a tree's node capacity rewrites every block of that
/// tree, so it is charged the result tree's whole entry count. This is the
/// part the result's headers count: its records where the root's capacity
/// changed, and its free entries where free space, which carries its own
/// capacity, changed that.
pub(super) fn whole_tree_rewrite(source: TreeHeaders, result: TreeHeaders) -> Option<usize> {
    let mut entries = 0_u64;
    if source.root_node_capacity != result.root_node_capacity {
        entries = entries.checked_add(result.routes)?;
    }
    if source.free_node_capacity != result.free_node_capacity {
        entries = entries.checked_add(result.free_entries)?;
    }
    usize::try_from(entries).ok()
}

/// The segment tree takes the root's node capacity, so a step that changed
/// it rewrote that tree whole as well. `result_segment_pages` is counted from
/// the tree itself.
pub(super) const fn segment_tree_rewrite(
    source: TreeHeaders,
    result: TreeHeaders,
    result_segment_pages: usize,
) -> usize {
    if source.root_node_capacity != result.root_node_capacity {
        result_segment_pages
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(root_node_capacity: u16, free_node_capacity: u16) -> TreeHeaders {
        TreeHeaders {
            root_node_capacity,
            free_node_capacity,
            routes: 100,
            free_entries: 3,
        }
    }

    #[test]
    fn a_member_step_is_charged_one_and_everything_its_member_declares() {
        assert_eq!(declared(0, 0, 0, 0), Some(1));
        assert_eq!(declared(2, 0, 0, 0), Some(3));
        assert_eq!(declared(0, 3, 0, 0), Some(4));
        assert_eq!(declared(0, 0, 5, 0), Some(6));
        assert_eq!(declared(0, 0, 0, 7), Some(8));
        assert_eq!(declared(2, 3, 5, 7), Some(18));
        assert_eq!(declared(usize::MAX, 0, 0, 0), None);
    }

    #[test]
    fn a_retirement_edge_without_a_member_is_charged_its_root() {
        assert_eq!(step(None), Some(1));
    }

    fn free(owner: u64, first_unallocated: u64) -> RecordFreeSpaceManifestEntry {
        RecordFreeSpaceManifestEntry::inline_frontier(owner, first_unallocated, 2, 8).unwrap()
    }

    #[test]
    fn a_retirement_edge_is_charged_the_free_entries_only_one_side_holds() {
        let source = [free(1, 1), free(2, 1), free(4, 1)];
        assert_eq!(net_free_difference(&source, &source), Some(0));
        assert_eq!(net_free_difference(&source, &[]), Some(3));
        assert_eq!(net_free_difference(&[], &source), Some(3));
        // One added in the middle, one dropped from the end.
        let result = [free(1, 1), free(2, 1), free(3, 1)];
        assert_eq!(net_free_difference(&source, &result), Some(2));
        // The same key with other contents left one side and joined the other.
        let rewritten = [free(1, 1), free(2, 7), free(4, 1)];
        assert_eq!(net_free_difference(&source, &rewritten), Some(2));
        // One only the source holds sorts before one only the result holds.
        let (before, after) = ([free(1, 1), free(3, 1)], [free(2, 1), free(3, 1)]);
        assert_eq!(net_free_difference(&before, &after), Some(2));
    }

    #[test]
    fn a_whole_tree_is_counted_from_the_headers_of_its_result() {
        use super::super::test_inventory::{inventory, root};
        let source = TreeHeaders::of(&root(64, 2), &inventory(64, 1, 1).free_space);
        let (result_root, result_inventory) = (root(128, 5), inventory(128, 3, 7));
        let result = TreeHeaders::of(&result_root, &result_inventory.free_space);
        assert_eq!(
            result,
            TreeHeaders {
                root_node_capacity: 128,
                free_node_capacity: 128,
                routes: 5,
                free_entries: 7,
            }
        );
        // Both capacities changed: every entry the result's headers count,
        // none of the source's.
        assert_eq!(whole_tree_rewrite(source, result), Some(12));
        assert_eq!(whole_tree_rewrite(result, source), Some(3));
    }

    #[test]
    fn only_a_changed_node_capacity_charges_a_whole_tree() {
        let source = headers(64, 64);
        assert_eq!(whole_tree_rewrite(source, headers(64, 64)), Some(0));
        assert_eq!(whole_tree_rewrite(source, headers(128, 64)), Some(100));
        assert_eq!(whole_tree_rewrite(source, headers(64, 128)), Some(3));
        assert_eq!(whole_tree_rewrite(source, headers(128, 128)), Some(103));
        let overflowing = TreeHeaders {
            routes: u64::MAX,
            ..headers(128, 128)
        };
        assert_eq!(whole_tree_rewrite(source, overflowing), None);
    }

    #[test]
    fn the_segment_tree_is_charged_whole_only_where_the_roots_capacity_changed() {
        let source = headers(64, 64);
        assert_eq!(segment_tree_rewrite(source, headers(64, 64), 20), 0);
        assert_eq!(segment_tree_rewrite(source, headers(64, 128), 20), 0);
        assert_eq!(segment_tree_rewrite(source, headers(128, 64), 20), 20);
        assert_eq!(segment_tree_rewrite(source, headers(128, 128), 20), 20);
    }
}
