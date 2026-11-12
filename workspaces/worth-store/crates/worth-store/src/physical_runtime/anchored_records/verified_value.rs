//! Reads the record an anchor selects and verifies it whole: frame, member
//! table, payload, the record digest the anchor carries and the payload digest
//! the frame carries.
//!
//! Reading streams. The member table grows only with bytes the record really
//! holds, and a payload read for verification alone is hashed in bounded
//! chunks without being retained, so a damaged length can neither allocate
//! nor be mistaken for absence. Cost is O(record bytes).

mod stream;
pub(super) use stream::AnchoredValueStream;

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
    pub(super) memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Reads and verifies the record `anchor` selects for `name`.
pub(super) fn read_verified(
    reader: &PhysicalRecordReader,
    name: AnchoredRecordName,
    anchor: RecordAnchor,
    retention: PayloadRetention,
) -> Result<VerifiedAnchoredValue, Failure> {
    read_verified_admitted(reader, name, anchor, retention, None)
}

pub(super) fn read_verified_membership(
    reader: &PhysicalRecordReader,
    name: AnchoredRecordName,
    anchor: RecordAnchor,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<VerifiedAnchoredValue, Failure> {
    read_verified_admitted(
        reader,
        name,
        anchor,
        PayloadRetention::DigestOnly,
        Some(request),
    )
}

fn read_verified_admitted(
    reader: &PhysicalRecordReader,
    name: AnchoredRecordName,
    anchor: RecordAnchor,
    retention: PayloadRetention,
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<VerifiedAnchoredValue, Failure> {
    let limit = match retention {
        PayloadRetention::Retain { maximum_bytes } => Some(maximum_bytes),
        PayloadRetention::DigestOnly => None,
    };
    let mut stream = AnchoredValueStream::open(reader, name, anchor, limit, request)?;
    let mut retained = match retention {
        PayloadRetention::Retain { .. } => {
            Some(Vec::with_capacity(stream.header.payload_bytes as usize))
        }
        PayloadRetention::DigestOnly => None,
    };
    let mut scratch = [0; PAYLOAD_CHUNK_BYTES];
    loop {
        let count = stream.read_next(request, &mut scratch)?;
        if count == 0 {
            break;
        }
        if let Some(payload) = &mut retained {
            payload.extend_from_slice(&scratch[..count]);
        }
    }
    Ok(stream.into_verified(retained))
}

/// Reads exactly `header.member_count` table entries in bounded chunks.
fn read_member_table(
    session: &mut RecordReadSession,
    header: &AnchoredRecordHeader,
    record_digest: &mut Sha256,
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<Vec<AnchoredRecordMemberLocator>, Failure> {
    let mut members = if request.is_some() {
        Vec::with_capacity(header.member_count as usize)
    } else {
        Vec::new()
    };
    let mut remaining = header.member_count as usize;
    let mut chunk = [0; TABLE_CHUNK_MEMBERS * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
    while remaining > 0 {
        if let Some(request) = request {
            request.check_live().map_err(|stop| {
                Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
            })?;
        }
        let entries = remaining.min(TABLE_CHUNK_MEMBERS);
        let bytes = &mut chunk[..entries * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
        require_filled(session, bytes)?;
        record_digest.update(&*bytes);
        for entry in bytes
            .as_chunks::<ANCHORED_RECORD_MEMBER_LOCATOR_BYTES>()
            .0
            .iter()
        {
            members.push(decode_member(entry).map_err(Failure::Damaged)?);
        }
        remaining -= entries;
    }
    Ok(members)
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
