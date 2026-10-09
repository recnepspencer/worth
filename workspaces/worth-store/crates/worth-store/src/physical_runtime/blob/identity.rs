use worth_store_physical_format::store_namespace::StableStoreIdentity;

/// Opaque Store-issued physical object identity, not a semantic content ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobObjectId {
    bytes: [u8; 16],
    store: StableStoreIdentity,
}

/// One Store-issued ingest attempt. It is never derived from object content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobSessionId([u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobGeneration(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishedBlobGeneration {
    object: BlobObjectId,
    generation: BlobGeneration,
    session: BlobSessionId,
    store: StableStoreIdentity,
}

impl BlobObjectId {
    pub const fn bytes(self) -> [u8; 16] {
        self.bytes
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub(super) const fn from_selected(store: StableStoreIdentity, bytes: [u8; 16]) -> Self {
        Self { bytes, store }
    }
}

impl BlobSessionId {
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }

    pub(super) const fn from_selected(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl BlobGeneration {
    pub const fn sequence(self) -> u64 {
        self.0
    }

    pub(super) const fn published(sequence: u64) -> Self {
        Self(sequence)
    }
}

impl PublishedBlobGeneration {
    pub const fn object(self) -> BlobObjectId {
        self.object
    }

    pub const fn generation(self) -> BlobGeneration {
        self.generation
    }

    pub const fn session(self) -> BlobSessionId {
        self.session
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub(super) const fn from_completed_publication(
        store: StableStoreIdentity,
        session: BlobSessionId,
        object: BlobObjectId,
        generation: BlobGeneration,
    ) -> Self {
        Self {
            object,
            generation,
            session,
            store,
        }
    }
}

pub(in crate::physical_runtime) fn random_nonzero_identity() -> Option<[u8; 16]> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).ok()?;
    (bytes != [0; 16]).then_some(bytes)
}
