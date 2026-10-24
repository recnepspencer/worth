use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchor};

use super::frame::{AnchoredRecordHeader, HEADER_BYTES};
use super::{
    AnchoredRecord, AnchoredRecordDamage, AnchoredRecordName, AnchoredRecordReadFailure,
    AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits, RecordReadSession,
};

type Failure = AnchoredRecordReadFailure;

impl AnchoredRecords<'_> {
    /// The current value of `name`, or `None` when none was ever published.
    ///
    /// The payload is verified against its frame and against the record digest
    /// the anchor carries; damage refuses and is never reported as absence. A
    /// payload larger than `maximum_payload_bytes` refuses before it is read.
    pub fn read(
        &self,
        name: AnchoredRecordName,
        maximum_payload_bytes: u64,
    ) -> Result<Option<AnchoredRecord>, Failure> {
        let reader = self.reader()?;
        let Some(anchor) = self.current_anchor(&reader, name) else {
            return Ok(None);
        };
        let mut session = open(&reader, anchor.record())?;
        let mut header_bytes = [0; HEADER_BYTES];
        require_filled(&mut session, &mut header_bytes)?;
        let header =
            AnchoredRecordHeader::decode(&header_bytes, name.anchor(), anchor.generation())
                .map_err(Failure::Damaged)?;
        let payload_length = usize::try_from(header.payload_bytes)
            .ok()
            .filter(|_| header.payload_bytes <= maximum_payload_bytes)
            .ok_or(Failure::LimitExceeded {
                payload_bytes: header.payload_bytes,
            })?;
        let mut payload = vec![0; payload_length];
        require_filled(&mut session, &mut payload)?;
        if fill(&mut session, &mut [0; 1])? != 0 {
            return Err(Failure::Damaged(AnchoredRecordDamage::TrailingBytes));
        }
        if header.record_sha256(&payload) != anchor.record_payload_sha256() {
            return Err(Failure::Damaged(AnchoredRecordDamage::RecordDigestMismatch));
        }
        header.verify_payload(&payload).map_err(Failure::Damaged)?;
        Ok(Some(AnchoredRecord {
            version: version(name, anchor, &header),
            bytes: payload,
        }))
    }

    pub(super) fn reader(&self) -> Result<PhysicalRecordReader, Failure> {
        self.runtime.records().map_err(Failure::Protection)
    }

    /// The anchor `name` selects in the reader's protected root.
    pub(super) fn current_anchor(
        &self,
        reader: &PhysicalRecordReader,
        name: AnchoredRecordName,
    ) -> Option<RecordAnchor> {
        reader.selected_record_anchors().get(name.anchor())
    }
}

/// The version an anchor selects, described by its verified header.
pub(super) fn version(
    name: AnchoredRecordName,
    anchor: RecordAnchor,
    header: &AnchoredRecordHeader,
) -> AnchoredRecordVersion {
    AnchoredRecordVersion {
        name,
        generation: anchor.generation(),
        record: PhysicalRecordId::from_persisted(anchor.record()),
        payload_sha256: header.payload_sha256,
        payload_bytes: header.payload_bytes,
    }
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

fn open(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<RecordReadSession, Failure> {
    let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
    reader
        .open(PhysicalRecordId::from_persisted(record), limit)
        .map_err(Failure::Open)
}
