//! Bounded tag-7 checkpoint ratchet for Store-attested released drops.
//! Decoding establishes shape only. C8 must join every selected route, C9
//! member and unique durable fate before treating either variant as custody.

mod accumulator;
mod accumulator_v2;
mod batch;
mod digest;
mod no_release;
mod tip;

pub use accumulator::{ReleaseCheckpointAccumulatorV1, RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES};
pub use accumulator_v2::{
    ReleaseCheckpointAccumulatorV2, RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
};
pub use batch::{ReleaseCheckpointBatchV1, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES};
pub use digest::{release_checkpoint_batch_records_digest_v1, ReleasedDropCumulativeEvidenceV1};
pub use no_release::{ReleaseCheckpointNoReleaseV1, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES};
pub use tip::{ReleasedDropTipProvenanceV1, RELEASE_CHECKPOINT_TIP_WIRE_BYTES};

use super::identity::{decode_identity, encode_identity};
use super::PhysicalCheckpointIdentity;
use crate::PersistedRecordIdentity;

const DOMAIN: &[u8] = b"store.physical.checkpoint.released-drop-custody.v1";
const DOMAIN_V2: &[u8] = b"store.physical.checkpoint.released-drop-custody.v2";
const VERSION: u8 = 1;
const VERSION_V2: u8 = 2;
const BATCH_KIND: u8 = 1;
const ACCUMULATOR_KIND: u8 = 2;
const NO_RELEASE_KIND: u8 = 3;
const PREFIX_BYTES: usize = 8 + DOMAIN.len() + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCheckpointCertificateDenial {
    Malformed,
    InvalidBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedDropWalFateWitnessV1 {
    lsn_start: u64,
    lsn_end_exclusive: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
}

impl ReleasedDropWalFateWitnessV1 {
    pub fn new(
        lsn_start: u64,
        lsn_end_exclusive: u64,
        identity_digest: [u8; 32],
        payload_digest: [u8; 32],
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        if lsn_start == 0
            || lsn_start >= lsn_end_exclusive
            || identity_digest == [0; 32]
            || payload_digest == [0; 32]
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            lsn_start,
            lsn_end_exclusive,
            identity_digest,
            payload_digest,
        })
    }

    pub const fn lsn_start(self) -> u64 {
        self.lsn_start
    }
    pub const fn lsn_end_exclusive(self) -> u64 {
        self.lsn_end_exclusive
    }
    pub const fn identity_digest(self) -> [u8; 32] {
        self.identity_digest
    }
    pub const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }

    fn write(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.lsn_start.to_le_bytes());
        out.extend_from_slice(&self.lsn_end_exclusive.to_le_bytes());
        out.extend_from_slice(&self.identity_digest);
        out.extend_from_slice(&self.payload_digest);
    }

    fn read(cursor: &mut Cursor<'_>) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        Self::new(
            u64::from_le_bytes(cursor.take()?),
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
            cursor.take()?,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCheckpointCertificateV1 {
    Batch(ReleaseCheckpointBatchV1),
    Accumulator(ReleaseCheckpointAccumulatorV1),
    AccumulatorV2(ReleaseCheckpointAccumulatorV2),
    NoRelease(ReleaseCheckpointNoReleaseV1),
}

impl ReleaseCheckpointCertificateV1 {
    pub fn encode(self) -> Vec<u8> {
        match self {
            Self::Batch(value) => value.encode(),
            Self::Accumulator(value) => value.encode(),
            Self::AccumulatorV2(value) => value.encode(),
            Self::NoRelease(value) => value.encode(),
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let mut cursor = Cursor::new(bytes);
        let (version, kind) = read_prefix(&mut cursor)?;
        match (version, kind) {
            (VERSION, BATCH_KIND) => ReleaseCheckpointBatchV1::read(&mut cursor).map(Self::Batch),
            (VERSION, ACCUMULATOR_KIND) => {
                ReleaseCheckpointAccumulatorV1::read(&mut cursor).map(Self::Accumulator)
            }
            (VERSION_V2, ACCUMULATOR_KIND) => {
                ReleaseCheckpointAccumulatorV2::read(&mut cursor).map(Self::AccumulatorV2)
            }
            (VERSION, NO_RELEASE_KIND) => {
                ReleaseCheckpointNoReleaseV1::read(&mut cursor).map(Self::NoRelease)
            }
            _ => Err(ReleaseCheckpointCertificateDenial::Malformed),
        }
        .and_then(|value| {
            if cursor.done() {
                Ok(value)
            } else {
                Err(ReleaseCheckpointCertificateDenial::Malformed)
            }
        })
    }
}

fn write_prefix(out: &mut Vec<u8>, kind: u8) {
    out.extend_from_slice(&(DOMAIN.len() as u64).to_le_bytes());
    out.extend_from_slice(DOMAIN);
    out.extend_from_slice(&[VERSION, kind]);
}

fn write_prefix_v2(out: &mut Vec<u8>, kind: u8) {
    out.extend_from_slice(&(DOMAIN_V2.len() as u64).to_le_bytes());
    out.extend_from_slice(DOMAIN_V2);
    out.extend_from_slice(&[VERSION_V2, kind]);
}

fn read_prefix(cursor: &mut Cursor<'_>) -> Result<(u8, u8), ReleaseCheckpointCertificateDenial> {
    let domain_len = u64::from_le_bytes(cursor.take()?);
    if domain_len != DOMAIN.len() as u64 {
        return Err(ReleaseCheckpointCertificateDenial::Malformed);
    }
    let domain = cursor.take_slice(DOMAIN.len())?;
    let version = cursor.take::<1>()?[0];
    if (domain != DOMAIN || version != VERSION) && (domain != DOMAIN_V2 || version != VERSION_V2) {
        return Err(ReleaseCheckpointCertificateDenial::Malformed);
    }
    Ok((version, cursor.take::<1>()?[0]))
}

fn write_checkpoint(out: &mut Vec<u8>, checkpoint: PhysicalCheckpointIdentity) {
    let mut identity = [0; 24];
    encode_identity(&mut identity, checkpoint);
    out.extend_from_slice(&identity);
}

fn read_checkpoint(
    cursor: &mut Cursor<'_>,
) -> Result<PhysicalCheckpointIdentity, ReleaseCheckpointCertificateDenial> {
    decode_identity(cursor.take_slice(24)?)
        .map_err(|_| ReleaseCheckpointCertificateDenial::Malformed)
}

fn write_record(out: &mut Vec<u8>, identity: PersistedRecordIdentity) {
    out.extend_from_slice(&identity.allocation_epoch());
    out.extend_from_slice(&identity.ordinal().to_le_bytes());
}

fn read_record(
    cursor: &mut Cursor<'_>,
) -> Result<PersistedRecordIdentity, ReleaseCheckpointCertificateDenial> {
    PersistedRecordIdentity::new(cursor.take()?, u64::from_le_bytes(cursor.take()?))
        .ok_or(ReleaseCheckpointCertificateDenial::Malformed)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take<const N: usize>(&mut self) -> Result<[u8; N], ReleaseCheckpointCertificateDenial> {
        let slice = self.take_slice(N)?;
        Ok(slice.try_into().expect("exact checked length"))
    }
    fn take_slice(&mut self, len: usize) -> Result<&'a [u8], ReleaseCheckpointCertificateDenial> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(ReleaseCheckpointCertificateDenial::Malformed)?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or(ReleaseCheckpointCertificateDenial::Malformed)?;
        self.offset = end;
        Ok(result)
    }
    fn done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
#[path = "tests/release_certificate.rs"]
mod tests;
