use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    durable_artifact_checksum, ManifestBlockReference, PhysicalRecordFormatDeclaration,
    RootRoutingBlockScopeIdentity,
};

use super::{
    scope::encode_durable_artifact_scope_prefix, PhysicalArtifactScope,
    PhysicalArtifactScopeIdentity,
};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn root_routing_block(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        identity: RootRoutingBlockScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::RootRoutingBlock {
                record_format,
                identity,
            },
            range,
        )
    }

    pub const fn root_routing_block_identity(self) -> Option<RootRoutingBlockScopeIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::RootRoutingBlock { identity, .. } => Some(identity),
            _ => None,
        }
    }

    pub(crate) fn root_routing_exact_scope_digest(self) -> u32 {
        let identity = self
            .root_routing_block_identity()
            .expect("root-routing scope carries its canonical identity");
        let mut prefix = [0_u8; 43];
        encode_durable_artifact_scope_prefix(self, 5, &mut prefix);
        let mut bytes = [0_u8; 121];
        bytes[..43].copy_from_slice(&prefix);
        bytes[43..51].copy_from_slice(&identity.tree().get().to_le_bytes());
        encode_reference(identity.reference(), &mut bytes[51..]);
        durable_artifact_checksum(&bytes)
    }
}

fn encode_reference(reference: ManifestBlockReference, target: &mut [u8]) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[18..22].copy_from_slice(&reference.checksum().to_le_bytes());
    target[22..38].copy_from_slice(&reference.first().allocation_epoch());
    target[38..46].copy_from_slice(&reference.first().ordinal().to_le_bytes());
    target[46..62].copy_from_slice(&reference.last().allocation_epoch());
    target[62..70].copy_from_slice(&reference.last().ordinal().to_le_bytes());
}
