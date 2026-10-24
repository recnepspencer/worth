//! Reads the record an anchor selects and verifies it whole: frame, member
//! table, payload, the record digest the anchor carries and the payload digest
//! the frame carries.
//!
//! Reading streams. The member table grows only with bytes the record really
//! holds, and a payload read for verification alone is hashed in bounded
//! chunks without being retained, so a damaged length can neither allocate
//! nor be mistaken for absence. Cost is O(record bytes).

use sha2::{Digest, Sha256};
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchor};

use super::frame::{decode_member, AnchoredRecordHeader, HEADER_BYTES};
use super::{
    AnchoredRecordDamage, AnchoredRecordMemberLocator, AnchoredRecordName,
    AnchoredRecordReadFailure, ANCHORED_RECORD_MEMBER_LOCATOR_BYTES,
};
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits, RecordReadSession,
};

type Failure = AnchoredRecordReadFailure;

const TABLE_CHUNK_MEMBERS: usize = 256;
const PAYLOAD_CHUNK_BYTES: usize = 64 << 10;

/// What a verified read keeps of the payload.
#[derive(Debug, Clone, Copy)]
pub(super) enum PayloadRetention {
    /// Keep the payload; refuse one longer than this before reading it.
    Retain { maximum_bytes: u64 },
    /// Hash the payload for verification and keep none of it.
    DigestOnly,
}

/// An anchored record verified against its anchor.
pub(super) struct VerifiedAnchoredValue {
    pub(super) header: AnchoredRecordHeader,
    pub(super) members: Vec<AnchoredRecordMemberLocator>,
    /// Present exactly when the payload was retained.
    pub(super) payload: Option<Vec<u8>>,
}

/// Reads and verifies the record `anchor` selects for `name`.
pub(super) fn read_verified(
    reader: &PhysicalRecordReader,
    name: AnchoredRecordName,
    anchor: RecordAnchor,
    retention: PayloadRetention,
) -> Result<VerifiedAnchoredValue, Failure> {
    let mut session = open(reader, anchor.record())?;
    let mut record_digest = Sha256::new();
    let mut header_bytes = [0; HEADER_BYTES];
    require_filled(&mut session, &mut header_bytes)?;
    record_digest.update(header_bytes);
    let header = AnchoredRecordHeader::decode(&header_bytes, name.anchor(), anchor.generation())
        .map_err(Failure::Damaged)?;
    if let PayloadRetention::Retain { maximum_bytes } = retention {
        if header.payload_bytes > maximum_bytes || usize::try_from(header.payload_bytes).is_err() {
            return Err(Failure::LimitExceeded {
                payload_bytes: header.payload_bytes,
            });
        }
    }
    let members = read_member_table(&mut session, &header, &mut record_digest)?;
    let (payload, payload_sha256) =
        read_payload(&mut session, &header, retention, &mut record_digest)?;
    if fill(&mut session, &mut [0; 1])? != 0 {
        return Err(Failure::Damaged(AnchoredRecordDamage::TrailingBytes));
    }
    if <[u8; 32]>::from(record_digest.finalize()) != anchor.record_payload_sha256() {
        return Err(Failure::Damaged(AnchoredRecordDamage::RecordDigestMismatch));
    }
    header
        .verify_payload_digest(payload_sha256)
        .map_err(Failure::Damaged)?;
    Ok(VerifiedAnchoredValue {
        header,
        members,
        payload,
    })
}

/// Reads exactly `header.member_count` table entries in bounded chunks.
fn read_member_table(
    session: &mut RecordReadSession,
    header: &AnchoredRecordHeader,
    record_digest: &mut Sha256,
) -> Result<Vec<AnchoredRecordMemberLocator>, Failure> {
    let mut members = Vec::new();
    let mut remaining = header.member_count as usize;
    let mut chunk = vec![0; TABLE_CHUNK_MEMBERS * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
    while remaining > 0 {
        let entries = remaining.min(TABLE_CHUNK_MEMBERS);
        let bytes = &mut chunk[..entries * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
        require_filled(session, bytes)?;
        record_digest.update(&*bytes);
        for entry in bytes.chunks_exact(ANCHORED_RECORD_MEMBER_LOCATOR_BYTES) {
            let entry = entry.try_into().expect("exact chunk");
            members.push(decode_member(entry).map_err(Failure::Damaged)?);
        }
        remaining -= entries;
    }
    Ok(members)
}

/// Reads the payload through both digests, keeping it only when retained.
fn read_payload(
    session: &mut RecordReadSession,
    header: &AnchoredRecordHeader,
    retention: PayloadRetention,
    record_digest: &mut Sha256,
) -> Result<(Option<Vec<u8>>, [u8; 32]), Failure> {
    let mut payload_digest = Sha256::new();
    let mut retained = match retention {
        PayloadRetention::Retain { .. } => Some(Vec::with_capacity(header.payload_bytes as usize)),
        PayloadRetention::DigestOnly => None,
    };
    let mut chunk = vec![0; PAYLOAD_CHUNK_BYTES];
    let mut remaining = header.payload_bytes;
    while remaining > 0 {
        let length = remaining.min(PAYLOAD_CHUNK_BYTES as u64) as usize;
        let bytes = &mut chunk[..length];
        require_filled(session, bytes)?;
        record_digest.update(&*bytes);
        payload_digest.update(&*bytes);
        if let Some(payload) = retained.as_mut() {
            payload.extend_from_slice(bytes);
        }
        remaining -= length as u64;
    }
    Ok((retained, payload_digest.finalize().into()))
}

/// Fills `buffer` exactly or refuses the record as truncated.
fn require_filled(session: &mut RecordReadSession, buffer: &mut [u8]) -> Result<(), Failure> {
    if fill(session, buffer)? < buffer.len() {
        return Err(Failure::Damaged(AnchoredRecordDamage::Truncated));
    }
    Ok(())
}

/// Reads until `buffer` is full or the record ends; returns the bytes read.
fn fill(session: &mut RecordReadSession, buffer: &mut [u8]) -> Result<usize, Failure> {
    let mut filled = 0;
    while filled < buffer.len() {
        let read = session
            .read_next(&mut buffer[filled..])
            .map_err(Failure::Stream)?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    Ok(filled)
}

/// Opens a record for streaming with no limit beyond the record format's.
fn open(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<RecordReadSession, Failure> {
    let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
    reader
        .open(PhysicalRecordId::from_persisted(record), limit)
        .map_err(Failure::Open)
}
