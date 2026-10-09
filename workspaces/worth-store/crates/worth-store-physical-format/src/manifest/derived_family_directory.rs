use crate::PersistedRecordIdentity;

/// The authoritative publication whose selected bytes a derived directory has indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedThroughBlobPublication {
    root_generation: u64,
    record: PersistedRecordIdentity,
    encoded_digest: [u8; 32],
}

impl IndexedThroughBlobPublication {
    pub fn new(
        root_generation: u64,
        record: PersistedRecordIdentity,
        encoded_digest: [u8; 32],
    ) -> Option<Self> {
        (root_generation != 0).then_some(Self {
            root_generation,
            record,
            encoded_digest,
        })
    }

    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }

    pub const fn encoded_digest(self) -> [u8; 32] {
        self.encoded_digest
    }
}

/// A selected C.5 record naming the derived family roots and its source watermark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedFamilyRootDirectoryBinding {
    directory_record: PersistedRecordIdentity,
    indexed_through_blob_publication: Option<IndexedThroughBlobPublication>,
}

impl DerivedFamilyRootDirectoryBinding {
    pub const fn new(
        directory_record: PersistedRecordIdentity,
        indexed_through_blob_publication: Option<IndexedThroughBlobPublication>,
    ) -> Self {
        Self {
            directory_record,
            indexed_through_blob_publication,
        }
    }

    pub const fn directory_record(self) -> PersistedRecordIdentity {
        self.directory_record
    }

    pub const fn indexed_through_blob_publication(self) -> Option<IndexedThroughBlobPublication> {
        self.indexed_through_blob_publication
    }
}

pub(super) fn encode_root_binding(bytes: &mut [u8], binding: DerivedFamilyRootDirectoryBinding) {
    encode_record(&mut bytes[..24], binding.directory_record());
    if let Some(publication) = binding.indexed_through_blob_publication() {
        encode_publication(&mut bytes[24..88], publication);
    }
}

pub(super) fn decode_root_binding(bytes: &[u8]) -> Option<DerivedFamilyRootDirectoryBinding> {
    let directory_record = decode_record(&bytes[..24])?;
    let publication = if bytes[24..88] == [0; 64] {
        None
    } else {
        Some(decode_publication(&bytes[24..88])?)
    };
    Some(DerivedFamilyRootDirectoryBinding::new(
        directory_record,
        publication,
    ))
}

pub(super) fn encode_publication(bytes: &mut [u8], publication: IndexedThroughBlobPublication) {
    encode_record(&mut bytes[..24], publication.record());
    bytes[24..32].copy_from_slice(&publication.root_generation().to_le_bytes());
    bytes[32..64].copy_from_slice(&publication.encoded_digest());
}

pub(super) fn decode_publication(bytes: &[u8]) -> Option<IndexedThroughBlobPublication> {
    IndexedThroughBlobPublication::new(
        u64::from_le_bytes(bytes[24..32].try_into().ok()?),
        decode_record(&bytes[..24])?,
        bytes[32..64].try_into().ok()?,
    )
}

fn encode_record(bytes: &mut [u8], record: PersistedRecordIdentity) {
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
}

fn decode_record(bytes: &[u8]) -> Option<PersistedRecordIdentity> {
    PersistedRecordIdentity::new(
        bytes[..16].try_into().ok()?,
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
    )
}
