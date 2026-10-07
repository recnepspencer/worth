use std::{mem::size_of, sync::Arc};

use worth_foundational::PartitionIdentity;

use crate::report::ChargedBytes;

use super::{
    construction::{Built, FrontierEvent, ShapeNode, SubtreeSpec},
    edit::{deletion_depth, search_depth},
    Link, Node, ReductionTree,
};

impl<T: ChargedBytes, F> ChargedBytes for ReductionTree<T, F> {
    fn additional_charged_bytes(&self) -> u64 {
        self.root
            .as_ref()
            .map_or(0, |node| node.retained_bytes)
            .saturating_add(self.identity.additional_charged_bytes())
    }
}

impl<T, F> ReductionTree<T, F> {
    pub(crate) fn checked_subtree_result_bound(
        partitions: usize,
        max_value_bytes: u64,
    ) -> Option<u64> {
        node_ceiling::<T>(max_value_bytes)?.checked_mul(u64::try_from(partitions).ok()?)
    }

    /// Scratch and resulting tree bytes for checked full construction,
    /// excluding caller-owned inputs and captured reducer storage. Callers
    /// reserve this before executing. Three node sets cover the final tree
    /// and path-copy temporaries. The shape, monotone stack, evaluation stack,
    /// and completed-node slots are explicit. The BFS split plan, queue,
    /// bounded frontier, join list, and join-result map coexist with final
    /// nodes. Six value caps cover the parent join's canonical buffers and
    /// temporaries; each parallel task reserves its own six caps separately.
    pub fn checked_build_memory_bound(partitions: usize, max_value_bytes: u64) -> Option<u64> {
        let count = u64::try_from(partitions).ok()?;
        let entries = u64::try_from(size_of::<(PartitionIdentity, T)>())
            .ok()?
            .checked_mul(count)?;
        let shape = u64::try_from(size_of::<ShapeNode<T>>())
            .ok()?
            .checked_mul(count)?;
        let stack = u64::try_from(size_of::<usize>()).ok()?.checked_mul(count)?;
        let completed = u64::try_from(size_of::<Option<Built<T>>>())
            .ok()?
            .checked_mul(count)?;
        let traversal = u64::try_from(size_of::<(usize, bool)>())
            .ok()?
            .checked_mul(count)?
            .checked_mul(2)?;
        let frontier = u64::try_from(size_of::<SubtreeSpec>())
            .ok()?
            .checked_mul(count)?
            .checked_mul(3)?
            .checked_add(
                u64::try_from(size_of::<FrontierEvent>())
                    .ok()?
                    .checked_mul(count)?
                    .checked_mul(2)?,
            )?
            .checked_add(
                u64::try_from(size_of::<(SubtreeSpec, bool)>())
                    .ok()?
                    .checked_mul(count)?
                    .checked_mul(2)?,
            )?;
        let split_flags = count;
        // A BTreeMap node stores keys, child pointers, and allocator metadata.
        // This conservative per-entry ceiling covers its live join results.
        let join_map = 256_u64.checked_mul(count)?;
        node_ceiling::<T>(max_value_bytes)?
            .checked_mul(count)?
            .checked_mul(3)?
            .checked_add(entries)?
            .checked_add(shape)?
            .checked_add(stack)?
            .checked_add(completed)?
            .checked_add(traversal)?
            .checked_add(frontier)?
            .checked_add(split_flags)?
            .checked_add(join_map)?
            .checked_add(u64::try_from(size_of::<Arc<Node<T>>>() * 4).ok()?)?
            .checked_add(max_value_bytes.checked_mul(6)?)
    }

    /// Additional edit scratch beyond the retained tree and incoming value.
    pub fn checked_update_memory_bound(
        &self,
        partition: PartitionIdentity,
        max_value_bytes: u64,
    ) -> Option<u64> {
        let (depth, found) = search_depth(&self.root, partition);
        found
            .then(|| path_ceiling::<T>(depth, max_value_bytes))
            .flatten()
    }

    /// Additional edit scratch beyond the retained tree and incoming value.
    pub fn checked_insert_memory_bound(
        &self,
        partition: PartitionIdentity,
        max_value_bytes: u64,
    ) -> Option<u64> {
        let (depth, found) = search_depth(&self.root, partition);
        (!found)
            .then(|| path_ceiling::<T>(depth + 1, max_value_bytes))
            .flatten()
    }

    /// Additional edit scratch beyond the retained tree. Includes the
    /// Cartesian merge path of the deleted node's children.
    pub fn checked_delete_memory_bound(
        &self,
        partition: PartitionIdentity,
        max_value_bytes: u64,
    ) -> Option<u64> {
        deletion_depth(&self.root, partition)
            .and_then(|depth| path_ceiling::<T>(depth, max_value_bytes))
    }
}

pub(super) fn node_bytes<T: ChargedBytes>(
    value: &T,
    aggregate: &T,
    left: &Link<T>,
    right: &Link<T>,
) -> Option<u64> {
    let own = size_of::<Node<T>>().checked_add(2 * size_of::<usize>())?;
    u64::try_from(own)
        .ok()?
        .checked_add(value.additional_charged_bytes())?
        .checked_add(aggregate.additional_charged_bytes())?
        .checked_add(left.as_ref().map_or(0, |node| node.retained_bytes))?
        .checked_add(right.as_ref().map_or(0, |node| node.retained_bytes))
}

fn node_ceiling<T>(max_value_bytes: u64) -> Option<u64> {
    u64::try_from(size_of::<Node<T>>().checked_add(2 * size_of::<usize>())?)
        .ok()?
        .checked_add(max_value_bytes.checked_mul(2)?)
}

fn path_ceiling<T>(depth: usize, max_value_bytes: u64) -> Option<u64> {
    let staged = u64::try_from(depth).ok()?.checked_add(1)?.checked_mul(4)?;
    node_ceiling::<T>(max_value_bytes)?
        .checked_mul(staged)?
        .checked_add(max_value_bytes.checked_mul(6)?)
}
