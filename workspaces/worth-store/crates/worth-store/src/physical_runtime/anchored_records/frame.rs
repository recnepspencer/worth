//! The anchored record frame: a fixed header binding the owner payload and the
//! member table to the anchor name and generation they were published under,
//! then the member table, then the payload.
//!
//! The anchor already carries the digest of the whole record, so the frame is
//! not the integrity root. It makes a record self-identifying: a record read
//! under another anchor or another generation refuses as damage instead of
//! serving a foreign value, and the table and payload lengths are known before
//! either is read. The member table lists every member's locator in the
//! publisher's order; a member's key is its locator's SHA-256.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordAnchorName, RECORD_ANCHOR_NAME_BYTES};

use super::{
    AnchoredRecordDamage, AnchoredRecordMemberLocator, ANCHORED_RECORD_MEMBER_LOCATOR_BYTES,
    MAXIMUM_ANCHORED_RECORD_MEMBERS,
};

const MAGIC: [u8; 8] = *b"WSANREC2";
const NAME_AT: usize = MAGIC.len();
const GENERATION_AT: usize = NAME_AT + RECORD_ANCHOR_NAME_BYTES;
const PAYLOAD_SHA256_AT: usize = GENERATION_AT + 8;
const PAYLOAD_BYTES_AT: usize = PAYLOAD_SHA256_AT + 32;
const MEMBER_COUNT_AT: usize = PAYLOAD_BYTES_AT + 8;
pub(super) const HEADER_BYTES: usize = MEMBER_COUNT_AT + 8;

/// The decoded header of one anchored record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AnchoredRecordHeader {
    pub(super) anchor: RecordAnchorName,
    pub(super) generation: u64,
    pub(super) payload_sha256: [u8; 32],
    pub(super) payload_bytes: u64,
    pub(super) member_count: u64,
}

impl AnchoredRecordHeader {
    /// The header of `payload` with `member_count` members, published as
    /// `generation` of `anchor`.
    pub(super) fn describe(
        anchor: RecordAnchorName,
        generation: u64,
        payload: &[u8],
        member_count: usize,
    ) -> Self {
        Self {
            anchor,
            generation,
            payload_sha256: Sha256::digest(payload).into(),
            payload_bytes: payload.len() as u64,
            member_count: member_count as u64,
        }
    }

    pub(super) fn encode(&self) -> [u8; HEADER_BYTES] {
        let mut bytes = [0; HEADER_BYTES];
        bytes[..NAME_AT].copy_from_slice(&MAGIC);
        bytes[NAME_AT..GENERATION_AT].copy_from_slice(&padded(self.anchor));
        bytes[GENERATION_AT..PAYLOAD_SHA256_AT].copy_from_slice(&self.generation.to_le_bytes());
        bytes[PAYLOAD_SHA256_AT..PAYLOAD_BYTES_AT].copy_from_slice(&self.payload_sha256);
        bytes[PAYLOAD_BYTES_AT..MEMBER_COUNT_AT].copy_from_slice(&self.payload_bytes.to_le_bytes());
        bytes[MEMBER_COUNT_AT..].copy_from_slice(&self.member_count.to_le_bytes());
        bytes
    }

    /// The full record: header, member table, payload. `members` must hold
    /// exactly `member_count` locators.
    pub(super) fn encode_record(
        &self,
        members: &[AnchoredRecordMemberLocator],
        payload: &[u8],
    ) -> Vec<u8> {
        debug_assert_eq!(members.len() as u64, self.member_count);
        let mut record = Vec::with_capacity(
            HEADER_BYTES + members.len() * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES + payload.len(),
        );
        record.extend_from_slice(&self.encode());
        for member in members {
            record.extend_from_slice(&member.to_bytes());
        }
        record.extend_from_slice(payload);
        record
    }

    /// The length of the full record this header describes.
    pub(super) fn record_bytes(&self) -> u64 {
        HEADER_BYTES as u64
            + self.member_count * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES as u64
            + self.payload_bytes
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
        let member_count = field(MEMBER_COUNT_AT);
        if member_count > MAXIMUM_ANCHORED_RECORD_MEMBERS as u64 {
            return Err(AnchoredRecordDamage::MalformedMemberTable);
        }
        Ok(Self {
            anchor,
            generation,
            payload_sha256: bytes[PAYLOAD_SHA256_AT..PAYLOAD_BYTES_AT]
                .try_into()
                .expect("fixed field"),
            payload_bytes: field(PAYLOAD_BYTES_AT),
            member_count,
        })
    }

    pub(super) fn verify_payload_digest(
        &self,
        payload_sha256: [u8; 32],
    ) -> Result<(), AnchoredRecordDamage> {
        if payload_sha256 != self.payload_sha256 {
            return Err(AnchoredRecordDamage::PayloadDigestMismatch);
        }
        Ok(())
    }
}

/// Decodes one member table entry.
pub(super) fn decode_member(
    bytes: &[u8; ANCHORED_RECORD_MEMBER_LOCATOR_BYTES],
) -> Result<AnchoredRecordMemberLocator, AnchoredRecordDamage> {
    AnchoredRecordMemberLocator::from_bytes(bytes).ok_or(AnchoredRecordDamage::MalformedMemberTable)
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
