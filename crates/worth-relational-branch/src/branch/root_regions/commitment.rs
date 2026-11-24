//! Canonical commitment of the persistent region tree and its carried facts.
use super::*;
pub(super) fn empty_commitment() -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(b"worth.relational.branch-region.empty.v5\0").into()
}

pub(super) fn node_commitment(
    depth: u32,
    zero: Option<&Arc<RelationalPersistentRegionNode>>,
    one: Option<&Arc<RelationalPersistentRegionNode>>,
    leaf: Option<&RelationalPersistentRegionLeaf>,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-region.node.v5\0");
    digest.update(depth.to_be_bytes());
    digest.update(zero.map_or_else(empty_commitment, |node| node.commitment));
    digest.update(one.map_or_else(empty_commitment, |node| node.commitment));
    match leaf {
        Some(RelationalPersistentRegionLeaf::Present(region)) => {
            digest.update([1]);
            digest.update(region.region.partition_id.0.to_be_bytes());
            digest.update(region.region.content_digest);
            digest.update(region.region.directory.digest());
            digest.update((region.region.live_counts.entities as u64).to_be_bytes());
            digest.update((region.region.live_counts.relations as u64).to_be_bytes());
            digest.update(region.region.kind_commitment.head.bytes());
            digest.update(region.region.kind_commitment.history.bytes());
        }
        Some(RelationalPersistentRegionLeaf::Removed(partition_id)) => {
            digest.update([2]);
            digest.update(partition_id.partition_id.0.to_be_bytes());
        }
        None => digest.update([0]),
    }
    digest.finalize().into()
}
