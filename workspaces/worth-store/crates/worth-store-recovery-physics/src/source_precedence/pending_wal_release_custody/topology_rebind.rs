//! Checked C.8 topology attachment; Store still independently rewalks media.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, PhysicalRecordFormatDeclaration,
};

use super::{PendingWalReleaseCustodyDenial, VerifiedPendingWalReleaseCustody};
use crate::source_precedence::released_v3_inventory_transition::VerifiedReleasedV3InventoryTransition;

impl VerifiedPendingWalReleaseCustody {
    /// C.8 attaches only a fully checked source-to-result semantic inventory
    /// transition. The digests remain transcripts until Store independently
    /// streams and compares both inventories from admitted media.
    pub fn rebind_verified_topology(
        &mut self,
        source_free: &DurableFreeSpaceManifestHeader,
        published_free: &DurableFreeSpaceManifestHeader,
        transition: VerifiedReleasedV3InventoryTransition,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PendingWalReleaseCustodyDenial> {
        let source = transition.source_topology();
        let published = transition.result_topology();
        let root = self
            .published_root
            .as_ref()
            .ok_or(PendingWalReleaseCustodyDenial::PublishedRoot)?;
        if self.verified_transition.is_some()
            || source_free.tier_epoch_start() != published_free.tier_epoch_start()
            || self.source_root.tier_epoch_anchor() != root.tier_epoch_anchor()
            || self.source_root.tier_epoch_anchor().is_some()
                != source_free.tier_epoch_start().is_some()
            || <[u8; 32]>::from(Sha256::digest(source_free.encode(format)))
                != self.descriptor.custody().source_free_space_frame_sha256()
            || !source.matches_headers(&self.source_root, source_free, format)
            || !published.matches_headers(root, published_free, format)
        {
            return Err(PendingWalReleaseCustodyDenial::PublishedRoot);
        }
        self.verified_transition = Some(transition);
        Ok(())
    }
}
