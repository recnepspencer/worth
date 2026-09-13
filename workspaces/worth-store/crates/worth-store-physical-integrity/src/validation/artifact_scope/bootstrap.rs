use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{durable_artifact_checksum, PhysicalRecordFormatDeclaration};

use super::{
    scope::encode_durable_artifact_scope_prefix, PhysicalArtifactScope,
    PhysicalArtifactScopeIdentity,
};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn bootstrap_catalog(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::BootstrapCatalog(record_format),
            range,
        )
    }

    pub(crate) fn bootstrap_exact_scope_digest(self) -> u32 {
        let mut bytes = [0_u8; 43];
        encode_durable_artifact_scope_prefix(self, 4, &mut bytes);
        durable_artifact_checksum(&bytes)
    }

    pub(crate) const fn is_bootstrap_catalog(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::BootstrapCatalog(_)
        )
    }
}
