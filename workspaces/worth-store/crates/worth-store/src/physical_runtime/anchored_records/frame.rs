//! The anchored record frame: a fixed header binding the owner payload to the
//! anchor name and generation it was published under, then the payload.
//!
//! The anchor already carries the digest of the whole record, so the frame is
//! not the integrity root. It makes a record self-identifying: a record read
//! under another anchor or another generation refuses as damage instead of
//! serving a foreign value, and the payload length is known before the payload
//! is read.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordAnchorName, RECORD_ANCHOR_NAME_BYTES};

use super::AnchoredRecordDamage;

const MAGIC: [u8; 8] = *b"WSANREC1";
const NAME_AT: usize = MAGIC.len();
const GENERATION_AT: usize = NAME_AT + RECORD_ANCHOR_NAME_BYTES;
const PAYLOAD_SHA256_AT: usize = GENERATION_AT + 8;
const PAYLOAD_BYTES_AT: usize = PAYLOAD_SHA256_AT + 32;
pub(super) const HEADER_BYTES: usize = PAYLOAD_BYTES_AT + 8;

/// The decoded header of one anchored record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AnchoredRecordHeader {
    pub(super) anchor: RecordAnchorName,
    pub(super) generation: u64,
    pub(super) payload_sha256: [u8; 32],
    pub(super) payload_bytes: u64,
}

impl AnchoredRecordHeader {
    /// The header of `payload` published as `generation` of `anchor`.
    pub(super) fn describe(anchor: RecordAnchorName, generation: u64, payload: &[u8]) -> Self {
        Self {
            anchor,
            generation,
            payload_sha256: Sha256::digest(payload).into(),
            payload_bytes: payload.len() as u64,
        }
    }

    pub(super) fn encode(&self) -> [u8; HEADER_BYTES] {
        let mut bytes = [0; HEADER_BYTES];
        bytes[..NAME_AT].copy_from_slice(&MAGIC);
        bytes[NAME_AT..GENERATION_AT].copy_from_slice(&padded(self.anchor));
        bytes[GENERATION_AT..PAYLOAD_SHA256_AT].copy_from_slice(&self.generation.to_le_bytes());
        bytes[PAYLOAD_SHA256_AT..PAYLOAD_BYTES_AT].copy_from_slice(&self.payload_sha256);
        bytes[PAYLOAD_BYTES_AT..].copy_from_slice(&self.payload_bytes.to_le_bytes());
        bytes
    }

    /// Encodes the full record: header followed by `payload`.
    pub(super) fn encode_record(&self, payload: &[u8]) -> Vec<u8> {
        let mut record = Vec::with_capacity(HEADER_BYTES + payload.len());
        record.extend_from_slice(&self.encode());
        record.extend_from_slice(payload);
        record
    }

    /// SHA-256 of the full record, the digest its anchor selects it with.
    pub(super) fn record_sha256(&self, payload: &[u8]) -> [u8; 32] {
        let mut record = Sha256::new();
        record.update(self.encode());
        record.update(payload);
        record.finalize().into()
    }

    /// Decodes a header that must name `anchor` at `generation`. Every field
    /// is fixed width, so a decoded header re-encodes to the same bytes.
    pub(super) fn decode(
        bytes: &[u8; HEADER_BYTES],
        anchor: RecordAnchorName,
        generation: u64,
    ) -> Result<Self, AnchoredRecordDamage> {
        if bytes[..NAME_AT] != MAGIC {
            return Err(AnchoredRecordDamage::WrongMagic);
        }
        if bytes[NAME_AT..GENERATION_AT] != padded(anchor) {
            return Err(AnchoredRecordDamage::WrongAnchor);
        }
        let field = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("fixed"));
        if field(GENERATION_AT) != generation {
            return Err(AnchoredRecordDamage::WrongGeneration);
        }
        Ok(Self {
            anchor,
            generation,
            payload_sha256: bytes[PAYLOAD_SHA256_AT..PAYLOAD_BYTES_AT]
                .try_into()
                .expect("fixed field"),
            payload_bytes: field(PAYLOAD_BYTES_AT),
        })
    }

    pub(super) fn verify_payload(&self, payload: &[u8]) -> Result<(), AnchoredRecordDamage> {
        if payload.len() as u64 != self.payload_bytes {
            return Err(AnchoredRecordDamage::Truncated);
        }
        if <[u8; 32]>::from(Sha256::digest(payload)) != self.payload_sha256 {
            return Err(AnchoredRecordDamage::PayloadDigestMismatch);
        }
        Ok(())
    }
}

/// The zero-padded fixed-width form of an anchor name.
fn padded(anchor: RecordAnchorName) -> [u8; RECORD_ANCHOR_NAME_BYTES] {
    let mut bytes = [0; RECORD_ANCHOR_NAME_BYTES];
    let name = anchor.as_str().as_bytes();
    bytes[..name.len()].copy_from_slice(name);
    bytes
}

#[cfg(test)]
#[path = "frame/tests.rs"]
mod tests;
