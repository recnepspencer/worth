use sha2::{Digest, Sha256};

use crate::PersistedRecordIdentity;

use super::super::super::envelope::{canonical_frame_sha256, nonzero_16};
use super::super::super::{BlobRecordDenial, BlobRecordKind};
use super::super::basis::read_record;
use super::super::drop_set_manifest::DROPPED_DIGEST_DOMAIN;
use super::super::{BlobReclaimSourceBasisV1, BlobReclaimSourceKind, MAXIMUM_DROP_SET_RECORDS};
use super::{DropSetManifestV3, MINIMUM_PAYLOAD_BYTES, NEVER_RESERVED};

#[derive(Debug, Clone, Copy)]
enum DroppedIdentities<'a> {
    Encoded(&'a [u8]),
    Owned(&'a [PersistedRecordIdentity]),
}

/// Validated V3 manifest grammar without ownership of its variable-length
/// identity sequence. Its bytes are borrowed from a witnessed control frame.
#[derive(Debug, Clone, Copy)]
pub struct DropSetManifestV3View<'a> {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_basis: BlobReclaimSourceBasisV1,
    dropped_digest: [u8; 32],
    dropped: DroppedIdentities<'a>,
    count: u16,
    manifest_selected_generation: u64,
}

impl<'a> DropSetManifestV3View<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DropSetManifestV3 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(super) fn decode_payload(payload: &'a [u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() < MINIMUM_PAYLOAD_BYTES + 112 + 24 {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let kind = BlobReclaimSourceKind::decode(payload[32])?;
        let source_len = u16::from_le_bytes(payload[33..35].try_into().unwrap()) as usize;
        let source_end = 35usize
            .checked_add(source_len)
            .ok_or(BlobRecordDenial::LengthMismatch)?;
        let fixed_end = source_end
            .checked_add(43)
            .ok_or(BlobRecordDenial::LengthMismatch)?;
        if payload.len() < fixed_end {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let source_basis = BlobReclaimSourceBasisV1::decode(kind, &payload[35..source_end])?;
        let count = u16::from_le_bytes(payload[source_end..source_end + 2].try_into().unwrap());
        if count == 0 || usize::from(count) > MAXIMUM_DROP_SET_RECORDS {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        let dropped_start = source_end + 34;
        let dropped_end = dropped_start
            .checked_add(usize::from(count) * 24)
            .ok_or(BlobRecordDenial::LengthMismatch)?;
        if payload.len() != dropped_end + 9 || payload[dropped_end + 8] != NEVER_RESERVED {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let store = nonzero_16(payload[..16].try_into().unwrap())?;
        let reclaim_attempt = nonzero_16(payload[16..32].try_into().unwrap())?;
        if let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = source_basis {
            if source.publication().store() != store {
                return Err(BlobRecordDenial::InvalidDropSet);
            }
        }
        let bytes = &payload[dropped_start..dropped_end];
        let mut hash = Sha256::new();
        hash.update(DROPPED_DIGEST_DOMAIN);
        hash.update(count.to_le_bytes());
        let mut prior = None;
        for chunk in bytes.chunks_exact(24) {
            let record = read_record(chunk)?;
            if prior.is_some_and(|previous| previous >= record) {
                return Err(BlobRecordDenial::InvalidDropSet);
            }
            if let BlobReclaimSourceBasisV1::FailedIngest(basis) = source_basis {
                if record == basis.declaration_record() || record == basis.abandoned_record() {
                    return Err(BlobRecordDenial::InvalidDropSet);
                }
            }
            prior = Some(record);
            hash.update(chunk);
        }
        let dropped_digest: [u8; 32] = hash.finalize().into();
        if payload[source_end + 2..dropped_start] != dropped_digest {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        let manifest_selected_generation =
            u64::from_le_bytes(payload[dropped_end..dropped_end + 8].try_into().unwrap());
        if manifest_selected_generation == 0 {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            store,
            reclaim_attempt,
            source_basis,
            dropped_digest,
            dropped: DroppedIdentities::Encoded(bytes),
            count,
            manifest_selected_generation,
        })
    }

    pub fn from_owned(manifest: &'a DropSetManifestV3) -> Self {
        Self {
            store: manifest.store(),
            reclaim_attempt: manifest.reclaim_attempt(),
            source_basis: manifest.source_basis(),
            dropped_digest: manifest.dropped_digest(),
            dropped: DroppedIdentities::Owned(manifest.dropped()),
            count: manifest.count(),
            manifest_selected_generation: manifest.never_reserved_slot_generation(),
        }
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_basis(self) -> BlobReclaimSourceBasisV1 {
        self.source_basis
    }
    pub const fn source_kind(self) -> BlobReclaimSourceKind {
        self.source_basis.kind()
    }
    pub fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis.digest(self.store)
    }
    pub const fn count(self) -> u16 {
        self.count
    }
    pub const fn never_reserved_slot_generation(self) -> u64 {
        self.manifest_selected_generation
    }

    pub fn contains_record(self, record: PersistedRecordIdentity) -> bool {
        match self.dropped {
            DroppedIdentities::Owned(records) => records.binary_search(&record).is_ok(),
            DroppedIdentities::Encoded(bytes) => {
                let mut lower = 0;
                let mut upper = usize::from(self.count);
                while lower < upper {
                    let middle = lower + (upper - lower) / 2;
                    let identity = read_record(&bytes[middle * 24..(middle + 1) * 24])
                        .expect("validated identity");
                    match identity.cmp(&record) {
                        std::cmp::Ordering::Less => lower = middle + 1,
                        std::cmp::Ordering::Equal => return true,
                        std::cmp::Ordering::Greater => upper = middle,
                    }
                }
                false
            }
        }
    }

    pub fn canonical_frame_sha256(self) -> [u8; 32] {
        let payload_len =
            MINIMUM_PAYLOAD_BYTES + self.source_basis.encoded_len() + usize::from(self.count) * 24;
        canonical_frame_sha256(BlobRecordKind::DropSetManifestV3, payload_len, |emit| {
            self.visit_payload(emit)
        })
    }

    pub(super) fn visit_payload(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.store);
        emit(&self.reclaim_attempt);
        emit(&[self.source_kind() as u8]);
        emit(&(self.source_basis.encoded_len() as u16).to_le_bytes());
        self.source_basis.visit_bytes(emit);
        emit(&self.count.to_le_bytes());
        emit(&self.dropped_digest);
        match self.dropped {
            DroppedIdentities::Encoded(bytes) => emit(bytes),
            DroppedIdentities::Owned(records) => {
                for record in records {
                    emit(&record.allocation_epoch());
                    emit(&record.ordinal().to_le_bytes());
                }
            }
        }
        emit(&self.manifest_selected_generation.to_le_bytes());
        emit(&[NEVER_RESERVED]);
    }

    pub(super) fn collect_owned(self) -> Vec<PersistedRecordIdentity> {
        let mut records = Vec::with_capacity(usize::from(self.count));
        match self.dropped {
            DroppedIdentities::Encoded(bytes) => {
                for chunk in bytes.chunks_exact(24) {
                    records.push(read_record(chunk).expect("validated identity"));
                }
            }
            DroppedIdentities::Owned(existing) => records.extend_from_slice(existing),
        }
        records
    }
}
