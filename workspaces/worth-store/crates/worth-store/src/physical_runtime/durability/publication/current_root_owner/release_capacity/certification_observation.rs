//! What a certification journey may observe of the selected release-head
//! custody: the root that names the head tree, both rosters and the selected
//! transitions still pending for the next checkpoint.

use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use crate::physical_runtime::durability::PhysicalCurrentRootOwner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificationReleaseHeadObservation {
    root: DurablePhysicalRootManifest,
    free_space: DurableFreeSpaceManifestHeader,
    effective_heads: (u64, [u8; 32]),
    checkpoint_heads: (u64, [u8; 32]),
    pending_drops: usize,
    pending_retirements: usize,
}

impl CertificationReleaseHeadObservation {
    pub const fn root_generation(&self) -> u64 {
        self.root.generation()
    }

    pub const fn has_head_tree(&self) -> bool {
        self.root.release_custody_head_root().is_some()
    }

    /// Count and digest of the roster after every selected WAL transition.
    pub const fn effective_heads(&self) -> (u64, [u8; 32]) {
        self.effective_heads
    }

    /// Count and digest of the last namespace-durable checkpoint's roster.
    pub const fn checkpoint_heads(&self) -> (u64, [u8; 32]) {
        self.checkpoint_heads
    }

    pub const fn pending_drops(&self) -> usize {
        self.pending_drops
    }

    pub const fn pending_retirements(&self) -> usize {
        self.pending_retirements
    }

    /// Whether this root directly succeeds `source` and differs from it only
    /// where a head-tree transition must: the head root and the head block
    /// frontier, beside the publication cell every root carries, the
    /// generation of the free-space header and the root's checksum of it.
    pub fn is_head_only_successor_of(&self, source: &Self) -> bool {
        let (root, prior) = (&self.root, &source.root);
        root.generation() == prior.generation() + 1
            && root.release_custody_head_root() != prior.release_custody_head_root()
            && head_frontier_is_exactly_one_step(root, prior)
            && root.requires_maintenance_protocol() == prior.requires_maintenance_protocol()
            && root.tree_identity() == prior.tree_identity()
            && root.node_capacity() == prior.node_capacity()
            && root.record_count() == prior.record_count()
            && root.next_block() == prior.next_block()
            && root.next_segment_block() == prior.next_segment_block()
            && root.routing_root() == prior.routing_root()
            && root.segment_root() == prior.segment_root()
            && root.free_space_root() == prior.free_space_root()
            && root.derived_family_directory() == prior.derived_family_directory()
            && root.latest_blob_publication() == prior.latest_blob_publication()
            && root.latest_blob_quarantine() == prior.latest_blob_quarantine()
            && root.tier_epoch_anchor() == prior.tier_epoch_anchor()
            && root.last_inline_record() == prior.last_inline_record()
            && root.last_inline_segment() == prior.last_inline_segment()
            && self.same_free_space_as(source)
    }

    fn same_free_space_as(&self, source: &Self) -> bool {
        let (space, prior) = (&self.free_space, &source.free_space);
        space.tree_identity() == prior.tree_identity()
            && space.node_capacity() == prior.node_capacity()
            && space.segment_page_capacity() == prior.segment_page_capacity()
            && space.entry_count() == prior.entry_count()
            && space.next_segment() == prior.next_segment()
            && space.next_page() == prior.next_page()
            && space.next_extent() == prior.next_extent()
            && space.next_arena() == prior.next_arena()
            && space.tier_epoch_start() == prior.tier_epoch_start()
            && space.next_block() == prior.next_block()
            && space.arena_capacity() == prior.arena_capacity()
            && space.arena_alignment() == prior.arena_alignment()
            && space.root() == prior.root()
    }
}

/// The head tree is copy-on-write and written leaf to root. A remaining root
/// is therefore the last block its step allocated, at or past the source
/// frontier and stamped with the successor generation; an emptied tree
/// allocates nothing.
fn head_frontier_is_exactly_one_step(
    root: &DurablePhysicalRootManifest,
    prior: &DurablePhysicalRootManifest,
) -> bool {
    let (frontier, source) = (
        root.next_release_custody_head_block(),
        prior.next_release_custody_head_block(),
    );
    match root.release_custody_head_root() {
        Some(head) => {
            head.generation() == root.generation()
                && head.block() >= source
                && head.block().checked_add(1) == Some(frontier)
        }
        None => frontier == source,
    }
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn certification_release_head_observation(
        &self,
    ) -> Option<CertificationReleaseHeadObservation> {
        let state = self.lock_publication_state();
        let ledger = state.release_ledger.selected()?;
        let pending_drops = ledger.pending_drop_count();
        Some(CertificationReleaseHeadObservation {
            root: state.current_root.clone(),
            free_space: state.free_space.clone(),
            effective_heads: ledger.effective_heads.commitment().ok()?,
            checkpoint_heads: ledger.checkpoint_heads.commitment().ok()?,
            pending_drops,
            pending_retirements: ledger.pending_events.len() - pending_drops,
        })
    }
}

#[cfg(test)]
#[path = "certification_observation/tests.rs"]
mod tests;
