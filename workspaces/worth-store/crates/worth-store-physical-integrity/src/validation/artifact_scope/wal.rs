use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::wal_frame::wal_frame_v1_validation_digest;
use worth_store_physical_format::WalSegmentIdentity;

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn wal_frame(
        store: StableStoreIdentity,
        identity: WalSegmentIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::WalFrame(identity),
            range,
        )
    }

    pub const fn wal_segment_identity(self) -> Option<WalSegmentIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::WalFrame(identity) => Some(identity),
            _ => None,
        }
    }

    pub(crate) const fn is_wal_frame(self) -> bool {
        matches!(self.identity, PhysicalArtifactScopeIdentity::WalFrame(_))
    }

    pub(crate) fn exact_wal_scope_digest(self) -> [u8; 32] {
        const DOMAIN: &[u8; 40] = b"worth-store-wal-frame-integrity-scope-v1";
        let identity = self
            .wal_segment_identity()
            .expect("WAL exact-scope digest requires WAL scope");
        let mut preimage = [0_u8; 96];
        preimage[..40].copy_from_slice(DOMAIN);
        preimage[40..56].copy_from_slice(&self.store.bytes());
        preimage[56..64].copy_from_slice(&identity.segment().get().to_le_bytes());
        preimage[64..72].copy_from_slice(&identity.generation().get().to_le_bytes());
        preimage[72..80].copy_from_slice(&self.range.offset().to_le_bytes());
        preimage[80..88].copy_from_slice(&self.range.length().to_le_bytes());
        let version = self.format_version();
        preimage[88..90].copy_from_slice(&version.format_version().to_le_bytes());
        preimage[90..92].copy_from_slice(&version.envelope_schema().unwrap_or(0).to_le_bytes());
        wal_frame_v1_validation_digest(&preimage)
    }
}
