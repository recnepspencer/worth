use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{durable_artifact_checksum, PhysicalRecordFormatDeclaration};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeDenial, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn current_root_selector(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::CurrentRootSelector(record_format),
            range,
        )
    }

    pub const fn previous_root_selector(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::PreviousRootSelector(record_format),
            range,
        )
    }

    pub const fn root_manifest(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        generation: u64,
        range: PhysicalByteRange,
    ) -> Result<Self, PhysicalArtifactScopeDenial> {
        if generation == 0 {
            return Err(PhysicalArtifactScopeDenial::ZeroRootGeneration);
        }
        Ok(Self::new(
            store,
            PhysicalArtifactScopeIdentity::RootManifest {
                record_format,
                generation,
            },
            range,
        ))
    }

    pub const fn root_generation(self) -> Option<u64> {
        match self.identity {
            PhysicalArtifactScopeIdentity::RootManifest { generation, .. } => Some(generation),
            _ => None,
        }
    }

    pub(crate) const fn is_current_selector(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::CurrentRootSelector(_)
        )
    }

    pub(crate) const fn is_previous_selector(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::PreviousRootSelector(_)
        )
    }

    pub(crate) const fn is_root_manifest(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::RootManifest { .. }
        )
    }

    /// Phase 3 exact preimage retained byte-for-byte for admitted selector/root views.
    pub(crate) fn selector_or_manifest_exact_scope_digest(self) -> u32 {
        let mut bytes = [0_u8; 51];
        bytes[..16].copy_from_slice(&self.store.bytes());
        bytes[16] = if self.is_current_selector() {
            1
        } else if self.is_previous_selector() {
            2
        } else if self.is_root_manifest() {
            3
        } else {
            panic!("exact scope digest is owned by Phase 3 root artifacts")
        };
        bytes[17..25].copy_from_slice(&self.root_generation().unwrap_or(0).to_le_bytes());
        bytes[25..33].copy_from_slice(&self.range.offset().to_le_bytes());
        bytes[33..41].copy_from_slice(&self.range.length().to_le_bytes());
        bytes[41..].copy_from_slice(&self.record_format().canonical_identity_bytes());
        durable_artifact_checksum(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::{PhysicalArtifactScope, PhysicalArtifactScopeDenial};
    use crate::localization::PhysicalByteRange;

    fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
        )
        .published_identity()
    }

    fn format() -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap()
    }

    #[test]
    fn selector_scope_does_not_invent_payload_only_root_generation() {
        let scope = PhysicalArtifactScope::current_root_selector(
            store(),
            format(),
            PhysicalByteRange::new(0, 107).unwrap(),
        );
        assert_eq!(
            scope.artifact_family(),
            PhysicalIntegrityArtifactFamily::CurrentRootSelector
        );
        assert_eq!(scope.root_generation(), None);
    }

    #[test]
    fn root_manifest_scope_requires_exact_nonzero_generation() {
        let range = PhysicalByteRange::new(0, 368).unwrap();
        assert_eq!(
            PhysicalArtifactScope::root_manifest(store(), format(), 0, range),
            Err(PhysicalArtifactScopeDenial::ZeroRootGeneration)
        );
        assert_eq!(
            PhysicalArtifactScope::root_manifest(store(), format(), 9, range)
                .unwrap()
                .root_generation(),
            Some(9)
        );
    }

    #[test]
    fn exact_scope_digest_changes_with_format_generation_or_range() {
        use worth_store_physical_format::PhysicalPageSizeClass;

        let range = PhysicalByteRange::new(4096, 368).unwrap();
        let baseline = PhysicalArtifactScope::root_manifest(store(), format(), 9, range).unwrap();
        let other_format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB32)
            .admit()
            .unwrap();

        assert_eq!(
            baseline.selector_or_manifest_exact_scope_digest(),
            1_585_574_697
        );
        assert_ne!(
            baseline.selector_or_manifest_exact_scope_digest(),
            PhysicalArtifactScope::root_manifest(store(), other_format, 9, range)
                .unwrap()
                .selector_or_manifest_exact_scope_digest()
        );
        assert_ne!(
            baseline.selector_or_manifest_exact_scope_digest(),
            PhysicalArtifactScope::root_manifest(store(), format(), 10, range)
                .unwrap()
                .selector_or_manifest_exact_scope_digest()
        );
        assert_ne!(
            baseline.selector_or_manifest_exact_scope_digest(),
            PhysicalArtifactScope::root_manifest(
                store(),
                format(),
                9,
                PhysicalByteRange::new(8192, 368).unwrap(),
            )
            .unwrap()
            .selector_or_manifest_exact_scope_digest()
        );
    }
}
