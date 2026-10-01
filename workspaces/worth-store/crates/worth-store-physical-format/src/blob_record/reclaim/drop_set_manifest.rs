use sha2::{Digest, Sha256};

use crate::PersistedRecordIdentity;

use super::super::envelope::{encode, nonzero_16};
use super::super::{BlobRecordDenial, BlobRecordKind};
use super::basis::{read_record, write_record};
use super::FailedIngestReclaimBasisV1;

pub const MAXIMUM_DROP_SET_RECORDS: usize = 1024;
const FIXED_PAYLOAD_BYTES: usize = 194;
pub(super) const DROPPED_DIGEST_DOMAIN: &[u8] = b"store.physical.blob-drop-set-identities.v1";

/// A routed custody record. Its existence does not authorize un-routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropSetManifestV1 {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_basis: FailedIngestReclaimBasisV1,
    dropped_digest: [u8; 32],
    dropped: Box<[PersistedRecordIdentity]>,
}

impl DropSetManifestV1 {
    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        source_basis: FailedIngestReclaimBasisV1,
        dropped: Vec<PersistedRecordIdentity>,
    ) -> Result<Self, BlobRecordDenial> {
        if dropped.is_empty()
            || dropped.len() > MAXIMUM_DROP_SET_RECORDS
            || dropped.windows(2).any(|pair| pair[0] >= pair[1])
            || dropped.contains(&source_basis.declaration_record())
            || dropped.contains(&source_basis.abandoned_record())
        {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            reclaim_attempt: nonzero_16(reclaim_attempt)?,
            source_basis,
            dropped_digest: digest_dropped(&dropped),
            dropped: dropped.into_boxed_slice(),
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        encode(BlobRecordKind::DropSetManifest, &self.encode_payload())
            .expect("bounded drop set fits control-frame ceiling")
    }

    pub(in crate::blob_record) fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(FIXED_PAYLOAD_BYTES + 24 * self.dropped.len());
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.reclaim_attempt);
        self.source_basis.encode_into(&mut payload);
        payload.extend_from_slice(&(self.dropped.len() as u16).to_le_bytes());
        payload.extend_from_slice(&self.dropped_digest);
        for record in &self.dropped {
            write_record(&mut payload, *record);
        }
        payload
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DropSetManifest {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() < FIXED_PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let count = u16::from_le_bytes(payload[160..162].try_into().expect("fixed count"));
        if count == 0 || usize::from(count) > MAXIMUM_DROP_SET_RECORDS {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        if payload.len() != FIXED_PAYLOAD_BYTES + usize::from(count) * 24 {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let mut dropped = Vec::with_capacity(usize::from(count));
        for bytes in payload[194..].chunks_exact(24) {
            dropped.push(read_record(bytes)?);
        }
        let value = Self::new(
            payload[..16].try_into().expect("fixed store"),
            payload[16..32].try_into().expect("fixed attempt"),
            FailedIngestReclaimBasisV1::decode(&payload[32..160])?,
            dropped,
        )?;
        if payload[162..194] != value.dropped_digest {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(value)
    }

    pub const fn store(&self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(&self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_basis(&self) -> FailedIngestReclaimBasisV1 {
        self.source_basis
    }
    pub fn source_basis_digest(&self) -> [u8; 32] {
        self.source_basis.digest(self.store)
    }
    pub const fn dropped_digest(&self) -> [u8; 32] {
        self.dropped_digest
    }
    pub fn dropped(&self) -> &[PersistedRecordIdentity] {
        &self.dropped
    }
    pub fn count(&self) -> u16 {
        self.dropped.len() as u16
    }
}

pub(super) fn digest_dropped(records: &[PersistedRecordIdentity]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(DROPPED_DIGEST_DOMAIN);
    hash.update((records.len() as u16).to_le_bytes());
    for record in records {
        hash.update(record.allocation_epoch());
        hash.update(record.ordinal().to_le_bytes());
    }
    hash.finalize().into()
}
