use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    durable_artifact_checksum, PhysicalRecordFormatDeclaration, SegmentManifestBlockReference,
    SegmentMembershipBlockScopeIdentity,
};

use super::{
    scope::encode_durable_artifact_scope_prefix, PhysicalArtifactScope,
    PhysicalArtifactScopeIdentity,
};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn segment_membership_block(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        identity: SegmentMembershipBlockScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::SegmentMembershipBlock {
                record_format,
                identity,
            },
            range,
        )
    }

    pub const fn segment_membership_block_identity(
        self,
    ) -> Option<SegmentMembershipBlockScopeIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::SegmentMembershipBlock { identity, .. } => {
                Some(identity)
            }
            _ => None,
        }
    }

    pub(crate) fn segment_membership_exact_scope_digest(self) -> u32 {
        let identity = self
            .segment_membership_block_identity()
            .expect("segment-membership scope carries its canonical identity");
        let mut prefix = [0_u8; 43];
        encode_durable_artifact_scope_prefix(self, 6, &mut prefix);
        let mut bytes = [0_u8; 105];
        bytes[..43].copy_from_slice(&prefix);
        bytes[43..51].copy_from_slice(&identity.tree().get().to_le_bytes());
        encode_reference(identity.reference(), &mut bytes[51..]);
        durable_artifact_checksum(&bytes)
    }
}

fn encode_reference(reference: SegmentManifestBlockReference, target: &mut [u8]) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[18..22].copy_from_slice(&reference.checksum().to_le_bytes());
    target[22..30].copy_from_slice(&reference.first().segment().get().to_le_bytes());
    target[30..38].copy_from_slice(&reference.first().page().get().to_le_bytes());
    target[38..46].copy_from_slice(&reference.last().segment().get().to_le_bytes());
    target[46..54].copy_from_slice(&reference.last().page().get().to_le_bytes());
}
