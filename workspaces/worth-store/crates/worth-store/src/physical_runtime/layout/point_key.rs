use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_layout_indexes::{BlobCatalogPointKey, BlobCatalogPointKeyDenial};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::BlobObjectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIndexPointKeyDenial {
    InvalidBlobCatalogKey(BlobCatalogPointKeyDenial),
}

/// Family-typed, canonical request identity. This is a request, not authority
/// to read a record or to assert a selected blob publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIndexPointKey {
    family: DurableArtifactFamilyId,
    store: StableStoreIdentity,
    blob_catalog: BlobCatalogPointKey,
}

impl PhysicalIndexPointKey {
    pub fn blob_catalog(
        object: BlobObjectId,
        generation: u64,
    ) -> Result<Self, PhysicalIndexPointKeyDenial> {
        Self::selected_blob_catalog(object.store(), object.bytes(), generation)
    }

    pub(in crate::physical_runtime) fn selected_blob_catalog(
        store: StableStoreIdentity,
        object: [u8; 16],
        generation: u64,
    ) -> Result<Self, PhysicalIndexPointKeyDenial> {
        let blob_catalog = BlobCatalogPointKey::admit(object, generation)
            .map_err(PhysicalIndexPointKeyDenial::InvalidBlobCatalogKey)?;
        Ok(Self {
            family: DurableArtifactFamilyId::BlobCatalog,
            store,
            blob_catalog,
        })
    }

    pub const fn family(self) -> DurableArtifactFamilyId {
        self.family
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn canonical_bytes(self) -> [u8; 24] {
        self.blob_catalog.bytes()
    }
}
