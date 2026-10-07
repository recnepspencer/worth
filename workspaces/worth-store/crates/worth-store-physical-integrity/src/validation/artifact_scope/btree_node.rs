use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{durable_artifact_checksum, PersistedRecordIdentity};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn btree_node(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        family_code: u16,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::BTreeNode {
                record,
                family_code,
            },
            range,
        )
    }

    pub const fn btree_node_identity(self) -> Option<(PersistedRecordIdentity, u16)> {
        match self.identity {
            PhysicalArtifactScopeIdentity::BTreeNode {
                record,
                family_code,
            } => Some((record, family_code)),
            _ => None,
        }
    }

    pub(crate) fn exact_btree_node_scope_digest(self) -> u32 {
        let (record, family_code) = self
            .btree_node_identity()
            .expect("B-tree scope digest requires a B-tree node");
        let mut bytes = [0_u8; 58];
        bytes[..16].copy_from_slice(&self.store.bytes());
        bytes[16..32].copy_from_slice(&record.allocation_epoch());
        bytes[32..40].copy_from_slice(&record.ordinal().to_le_bytes());
        bytes[40..42].copy_from_slice(&family_code.to_le_bytes());
        bytes[42..50].copy_from_slice(&self.range.offset().to_le_bytes());
        bytes[50..58].copy_from_slice(&self.range.length().to_le_bytes());
        durable_artifact_checksum(&bytes)
    }
}
