use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    durable_artifact_checksum, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalRecordFormatDeclaration,
};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn free_space_header(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        identity: FreeSpaceHeaderScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::FreeSpaceHeader {
                record_format,
                identity,
            },
            range,
        )
    }

    pub const fn free_space_membership_block(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        identity: FreeSpaceMembershipBlockScopeIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::FreeSpaceMembershipBlock {
                record_format,
                identity,
            },
            range,
        )
    }

    pub const fn free_space_header_identity(self) -> Option<FreeSpaceHeaderScopeIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::FreeSpaceHeader { identity, .. } => Some(identity),
            _ => None,
        }
    }

    pub const fn free_space_membership_block_identity(
        self,
    ) -> Option<FreeSpaceMembershipBlockScopeIdentity> {
        match self.identity {
            PhysicalArtifactScopeIdentity::FreeSpaceMembershipBlock { identity, .. } => {
                Some(identity)
            }
            _ => None,
        }
    }

    pub(crate) fn free_space_exact_scope_digest(self) -> Option<u32> {
        let mut bytes = [0_u8; 128];
        bytes[..16].copy_from_slice(&self.store.bytes());
        bytes[17..27].copy_from_slice(&self.record_format().canonical_identity_bytes());
        bytes[27..35].copy_from_slice(&self.range.offset().to_le_bytes());
        bytes[35..43].copy_from_slice(&self.range.length().to_le_bytes());
        match self.identity {
            PhysicalArtifactScopeIdentity::FreeSpaceHeader { identity, .. } => {
                bytes[16] = 1;
                bytes[43..51].copy_from_slice(&identity.generation().get().to_le_bytes());
                bytes[51..59].copy_from_slice(&identity.tree().get().to_le_bytes());
                bytes[59] = u8::from(identity.root().is_some());
                if let Some(root) = identity.root() {
                    encode_reference(&mut bytes[60..116], root);
                }
                bytes[116..120]
                    .copy_from_slice(&identity.complete_child_checksum().get().to_le_bytes());
            }
            PhysicalArtifactScopeIdentity::FreeSpaceMembershipBlock { identity, .. } => {
                bytes[16] = 2;
                bytes[43..51].copy_from_slice(&identity.tree().get().to_le_bytes());
                encode_reference(&mut bytes[51..107], identity.reference());
            }
            _ => return None,
        }
        Some(durable_artifact_checksum(&bytes))
    }
}

fn encode_reference(target: &mut [u8], reference: FreeSpaceBlockReference) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[20..24].copy_from_slice(&reference.checksum().to_le_bytes());
    encode_key(&mut target[24..40], reference.first());
    encode_key(&mut target[40..56], reference.last());
}

fn encode_key(target: &mut [u8], key: FreeSpaceKey) {
    target[0] = key.class() as u8;
    target[8..16].copy_from_slice(&key.owner().to_le_bytes());
}
