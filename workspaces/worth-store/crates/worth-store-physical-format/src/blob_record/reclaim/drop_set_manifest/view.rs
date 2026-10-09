use sha2::{Digest, Sha256};

use crate::PersistedRecordIdentity;

use super::super::super::envelope::nonzero_16;
use super::super::super::BlobRecordDenial;
use super::super::basis::read_record;
use super::super::FailedIngestReclaimBasisV1;
use super::{DROPPED_DIGEST_DOMAIN, FIXED_PAYLOAD_BYTES, MAXIMUM_DROP_SET_RECORDS};

/// Borrowed drop-set payload shared by the owned representation and current
/// failed-ingest manifest. This internal value admits no standalone wire kind.
#[derive(Debug, Clone, Copy)]
pub(in crate::blob_record::reclaim) struct DropSetManifestV1View<'a> {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_basis: FailedIngestReclaimBasisV1,
    dropped_digest: [u8; 32],
    dropped: &'a [u8],
    count: u16,
}

impl<'a> DropSetManifestV1View<'a> {
    pub(in crate::blob_record::reclaim) fn decode_payload(
        payload: &'a [u8],
    ) -> Result<Self, BlobRecordDenial> {
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
        let store = nonzero_16(payload[..16].try_into().expect("fixed store"))?;
        let reclaim_attempt = nonzero_16(payload[16..32].try_into().expect("fixed attempt"))?;
        let source_basis = FailedIngestReclaimBasisV1::decode(&payload[32..160])?;
        let dropped = &payload[FIXED_PAYLOAD_BYTES..];
        let mut hash = Sha256::new();
        hash.update(DROPPED_DIGEST_DOMAIN);
        hash.update(count.to_le_bytes());
        let mut prior = None;
        for chunk in dropped.chunks_exact(24) {
            let record = read_record(chunk)?;
            if prior.is_some_and(|previous| previous >= record)
                || record == source_basis.declaration_record()
                || record == source_basis.abandoned_record()
            {
                return Err(BlobRecordDenial::InvalidDropSet);
            }
            prior = Some(record);
            hash.update(chunk);
        }
        let dropped_digest: [u8; 32] = hash.finalize().into();
        if payload[162..194] != dropped_digest {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            store,
            reclaim_attempt,
            source_basis,
            dropped_digest,
            dropped,
            count,
        })
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_basis(self) -> FailedIngestReclaimBasisV1 {
        self.source_basis
    }
    pub fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis.digest(self.store)
    }
    pub const fn dropped_digest(self) -> [u8; 32] {
        self.dropped_digest
    }
    pub const fn count(self) -> u16 {
        self.count
    }

    pub fn contains_record(self, record: PersistedRecordIdentity) -> bool {
        let mut lower = 0;
        let mut upper = usize::from(self.count);
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            let identity = read_record(&self.dropped[middle * 24..(middle + 1) * 24])
                .expect("validated dropped identity");
            match identity.cmp(&record) {
                std::cmp::Ordering::Less => lower = middle + 1,
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Greater => upper = middle,
            }
        }
        false
    }

    pub(super) fn collect_owned(self) -> Vec<PersistedRecordIdentity> {
        self.dropped
            .chunks_exact(24)
            .map(|chunk| read_record(chunk).expect("validated dropped identity"))
            .collect()
    }
}
