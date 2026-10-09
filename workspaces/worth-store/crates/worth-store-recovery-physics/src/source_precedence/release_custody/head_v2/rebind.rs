//! Ordinary root publication preserves already-admitted checkpoint-source heads.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};

use super::{PhysicalSourceSelection, SelectedCustodyDenial, VerifiedSelectedReleaseHeadCustodyV2};

impl VerifiedSelectedReleaseHeadCustodyV2 {
    /// Rebinds only an ordinary, authenticated publication that leaves the
    /// release-head tree untouched. V14 root changes require the separate
    /// C.9-admitted replay token and cannot enter by this method.
    pub fn rebind_published_root(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), SelectedCustodyDenial> {
        let old = &self.roster.selected_root;
        if old != selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.tree_identity() != old.tree_identity()
            || published.tier_epoch_anchor() != old.tier_epoch_anchor()
            || published.generation() < old.generation()
            || (published.generation() == old.generation() && published != old)
            || published.release_custody_head_root() != old.release_custody_head_root()
            || published.next_release_custody_head_block() != old.next_release_custody_head_block()
            || published.generation() < self.roster.checkpoint_source_root.generation()
        {
            return Err(SelectedCustodyDenial::ReleaseBinding);
        }
        self.roster.selected_root_sha256 = Sha256::digest(published.encode(format)).into();
        self.roster.selected_root = published.clone();
        Ok(())
    }
}
