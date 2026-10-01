use super::{DurablePhysicalRootManifest, RootManifestDenial};
use crate::record_framing::{
    DurableFrameDenial, DIRECTORY_BOUND_MAINTENANCE_ROOT_SCHEMA, DIRECTORY_BOUND_ROOT_SCHEMA,
    MAINTENANCE_ROOT_SCHEMA, QUARANTINE_BOUND_MAINTENANCE_ROOT_SCHEMA,
    QUARANTINE_BOUND_ROOT_SCHEMA, TIER_ANCHORED_MAINTENANCE_ROOT_SCHEMA,
};

impl DurablePhysicalRootManifest {
    pub const fn requires_maintenance_protocol(&self) -> bool {
        self.requires_maintenance_protocol
    }

    pub fn with_maintenance_protocol(mut self) -> Self {
        self.requires_maintenance_protocol = true;
        self
    }

    pub(super) fn with_root_schema(mut self, schema: u8) -> Self {
        self.requires_maintenance_protocol = matches!(
            schema,
            MAINTENANCE_ROOT_SCHEMA
                | DIRECTORY_BOUND_MAINTENANCE_ROOT_SCHEMA
                | QUARANTINE_BOUND_MAINTENANCE_ROOT_SCHEMA
                | TIER_ANCHORED_MAINTENANCE_ROOT_SCHEMA
                | 10
        );
        self
    }

    /// C.9 envelope only. A maintenance-capable root is rejected before its fields are served.
    pub fn decode_c9_legacy(
        bytes: &[u8],
        max_entries: u16,
    ) -> Result<(Self, crate::PhysicalRecordFormatDeclaration), RootManifestDenial> {
        if bytes.get(9).is_some_and(|schema| {
            matches!(
                *schema,
                MAINTENANCE_ROOT_SCHEMA
                    | DIRECTORY_BOUND_ROOT_SCHEMA
                    | DIRECTORY_BOUND_MAINTENANCE_ROOT_SCHEMA
                    | QUARANTINE_BOUND_ROOT_SCHEMA
                    | QUARANTINE_BOUND_MAINTENANCE_ROOT_SCHEMA
                    | TIER_ANCHORED_MAINTENANCE_ROOT_SCHEMA
                    | 10
            )
        }) {
            return Err(RootManifestDenial::Frame(
                DurableFrameDenial::UnsupportedSchema(bytes[9]),
            ));
        }
        Self::decode(bytes, max_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::DurablePhysicalRootManifest;
    use crate::{
        DerivedFamilyRootDirectoryBinding, FreeSpaceBlockReference, FreeSpaceKey,
        IndexedThroughBlobPublication, ManifestBlockReference, PersistedRecordIdentity,
        PhysicalRecordFormatDeclaration,
    };

    fn manifest(maintenance: bool) -> DurablePhysicalRootManifest {
        let key = FreeSpaceKey::inline(1).unwrap();
        let free = FreeSpaceBlockReference::new(1, 1, 0, 41, key, key).unwrap();
        let root = DurablePhysicalRootManifest::builder(1, 9, 2, 43)
            .free_space_root(Some(free))
            .admit()
            .unwrap();
        if maintenance {
            root.with_maintenance_protocol()
        } else {
            root
        }
    }

    #[test]
    fn maintenance_root_is_a_distinct_envelope() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let legacy = manifest(false).encode(format);
        let maintained = manifest(true).encode(format);
        assert_eq!(legacy[9], 2);
        assert_eq!(maintained[9], 3);
        assert!(!DurablePhysicalRootManifest::decode(&legacy, u16::MAX)
            .unwrap()
            .0
            .requires_maintenance_protocol());
        assert!(DurablePhysicalRootManifest::decode(&maintained, u16::MAX)
            .unwrap()
            .0
            .requires_maintenance_protocol());
        assert!(DurablePhysicalRootManifest::decode_c9_legacy(&legacy, u16::MAX).is_ok());
        assert!(DurablePhysicalRootManifest::decode_c9_legacy(&maintained, u16::MAX).is_err());
    }

    #[test]
    fn extended_root_preserves_legacy_versions_and_exact_marker_binding() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let publication = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let directory = PersistedRecordIdentity::new([7; 16], 2).unwrap();
        let route = ManifestBlockReference::new(1, 1, 0, 31, publication, directory).unwrap();
        let marker = IndexedThroughBlobPublication::new(1, publication, [9; 32]).unwrap();
        let root = DurablePhysicalRootManifest::builder(1, 9, 2, 43)
            .record_count(2)
            .next_block(2)
            .routing_root(Some(route))
            .latest_blob_publication(Some(marker))
            .derived_family_directory(Some(DerivedFamilyRootDirectoryBinding::new(
                directory,
                Some(marker),
            )))
            .admit()
            .unwrap();
        for (candidate, schema) in [(root.clone(), 4), (root.with_maintenance_protocol(), 5)] {
            let bytes = candidate.encode(format);
            assert_eq!(bytes.len(), 544);
            assert_eq!(bytes[9], schema);
            assert_eq!(
                DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                    .unwrap()
                    .0,
                candidate
            );
            assert!(DurablePhysicalRootManifest::decode_c9_legacy(&bytes, u16::MAX).is_err());
        }
        assert_eq!(manifest(false).encode(format).len(), 384);
        assert_eq!(manifest(true).encode(format).len(), 384);
    }

    #[test]
    fn quarantined_root_binds_exact_selected_record_in_new_schema() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let publication = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let quarantine = PersistedRecordIdentity::new([7; 16], 2).unwrap();
        let route = ManifestBlockReference::new(1, 1, 0, 31, publication, quarantine).unwrap();
        let marker = IndexedThroughBlobPublication::new(1, publication, [9; 32]).unwrap();
        let root = DurablePhysicalRootManifest::builder(1, 9, 2, 43)
            .record_count(2)
            .next_block(2)
            .routing_root(Some(route))
            .latest_blob_publication(Some(marker))
            .latest_blob_quarantine(Some(quarantine))
            .admit()
            .unwrap();
        for (candidate, schema) in [(root.clone(), 6), (root.with_maintenance_protocol(), 7)] {
            let bytes = candidate.encode(format);
            assert_eq!(bytes.len(), 576);
            assert_eq!(bytes[9], schema);
            assert_eq!(
                DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                    .unwrap()
                    .0,
                candidate
            );
            assert!(DurablePhysicalRootManifest::decode_c9_legacy(&bytes, u16::MAX).is_err());
        }
    }

    #[test]
    fn tier_anchor_is_a_maintenance_only_root_binding() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
            .tier_epoch_anchor(Some([7; 32]))
            .admit()
            .unwrap();
        assert!(root.requires_maintenance_protocol());
        let bytes = root.encode(format);
        assert_eq!(bytes.len(), 608);
        assert_eq!(bytes[9], 9);
        assert_eq!(
            DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                .unwrap()
                .0,
            root
        );
        assert!(DurablePhysicalRootManifest::decode_c9_legacy(&bytes, u16::MAX).is_err());
        let altered = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
            .tier_epoch_anchor(Some([8; 32]))
            .admit()
            .unwrap();
        assert_ne!(bytes, altered.encode(format));
        assert!(DurablePhysicalRootManifest::builder(2, 9, 2, 43)
            .tier_epoch_anchor(Some([0; 32]))
            .admit()
            .is_none());
    }
}
