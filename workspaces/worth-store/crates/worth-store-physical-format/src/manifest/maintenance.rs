use super::{DurablePhysicalRootManifest, RootManifestDenial};
use crate::record_framing::{DurableFrameDenial, MAINTENANCE_ROOT_SCHEMA};

impl DurablePhysicalRootManifest {
    pub const fn requires_maintenance_protocol(&self) -> bool {
        self.requires_maintenance_protocol
    }

    pub fn with_maintenance_protocol(mut self) -> Self {
        self.requires_maintenance_protocol = true;
        self
    }

    pub(super) fn with_root_schema(mut self, schema: u8) -> Self {
        self.requires_maintenance_protocol = schema == MAINTENANCE_ROOT_SCHEMA;
        self
    }

    /// C.9 envelope only. A maintenance-capable root is rejected before its fields are served.
    pub fn decode_c9_legacy(
        bytes: &[u8],
        max_entries: u16,
    ) -> Result<(Self, crate::PhysicalRecordFormatDeclaration), RootManifestDenial> {
        if bytes.get(9) == Some(&MAINTENANCE_ROOT_SCHEMA) {
            return Err(RootManifestDenial::Frame(
                DurableFrameDenial::UnsupportedSchema(MAINTENANCE_ROOT_SCHEMA),
            ));
        }
        Self::decode(bytes, max_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::DurablePhysicalRootManifest;
    use crate::{
        FreeSpaceBlockReference, FreeSpaceKey, PhysicalRecordFormatDeclaration, RecordAllocationClass,
    };

    fn manifest(maintenance: bool) -> DurablePhysicalRootManifest {
        let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
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
}
