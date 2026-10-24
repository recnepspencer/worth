use sha2::{Digest, Sha256};
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchor};

use super::frame::{JournalRecordHeader, MAXIMUM_HEADER_BYTES};
use super::{
    CanonicalCommitEntry, CanonicalCommitJournal, CanonicalCommitPayload,
    CanonicalCommitReadFailure, CanonicalCommitResolution,
};
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

impl CanonicalCommitJournal<'_> {
    /// The newest durable entry, or `None` for an empty journal.
    pub fn head(&self) -> Result<Option<CanonicalCommitEntry>, CanonicalCommitReadFailure> {
        let reader = self.reader()?;
        match self.head_anchor(&reader) {
            None => Ok(None),
            Some(anchor) => self
                .entry_at(&reader, anchor, anchor.generation())
                .map(Some),
        }
    }

    /// Answers from durable state whether `position` holds `payload_sha256`.
    pub fn resolve(
        &self,
        position: u64,
        payload_sha256: [u8; 32],
    ) -> Result<CanonicalCommitResolution, CanonicalCommitReadFailure> {
        let unresolved = self.runtime.canonical_journal_state().unresolved;
        let reader = self.reader()?;
        let head = self
            .head_anchor(&reader)
            .map_or(0, RecordAnchor::generation);
        if position == 0 || position > head {
            return Ok(if unresolved == Some(position) {
                CanonicalCommitResolution::Unresolved
            } else {
                CanonicalCommitResolution::NotCommitted { head }
            });
        }
        let entry = self.locate(&reader, position)?;
        Ok(if entry.payload_sha256 == payload_sha256 {
            CanonicalCommitResolution::Committed(entry)
        } else {
            CanonicalCommitResolution::Conflict(entry)
        })
    }

    /// Reads one entry's payload, verified against its recorded digest.
    pub fn read(
        &self,
        position: u64,
        maximum_payload_bytes: u64,
    ) -> Result<CanonicalCommitPayload, CanonicalCommitReadFailure> {
        let reader = self.reader()?;
        let entry = self.locate(&reader, position)?;
        if entry.payload_bytes > maximum_payload_bytes {
            return Err(CanonicalCommitReadFailure::LimitExceeded {
                payload_bytes: entry.payload_bytes,
            });
        }
        let bytes = read_record(&reader, persisted(entry.record))?;
        let header = JournalRecordHeader::decode(&bytes, self.name, Some(bytes.len() as u64))
            .map_err(|_| CanonicalCommitReadFailure::Damaged)?;
        let payload = bytes[header.encoded_bytes()..].to_vec();
        if header.position != position || header.verify_payload(&payload).is_err() {
            return Err(CanonicalCommitReadFailure::Damaged);
        }
        Ok(CanonicalCommitPayload {
            entry,
            bytes: payload,
        })
    }

    /// Entries at or below `at_or_below`, newest first, at most `limit` of
    /// them. Resume with the position below the last entry returned.
    pub fn range_descending(
        &self,
        at_or_below: u64,
        limit: usize,
    ) -> Result<Vec<CanonicalCommitEntry>, CanonicalCommitReadFailure> {
        let reader = self.reader()?;
        let head = self
            .head_anchor(&reader)
            .map_or(0, RecordAnchor::generation);
        let start = at_or_below.min(head);
        let mut entries = Vec::new();
        if start == 0 || limit == 0 {
            return Ok(entries);
        }
        let mut entry = self.locate(&reader, start)?;
        loop {
            entries.push(entry);
            if entries.len() == limit || entry.position == 1 {
                return Ok(entries);
            }
            let header = read_header(&reader, self, persisted(entry.record))?;
            let previous = header
                .spine
                .first()
                .filter(|previous| previous.position + 1 == entry.position)
                .ok_or(CanonicalCommitReadFailure::Damaged)?;
            entry = self.entry_from(&reader, previous.record, previous.position)?;
        }
    }

    pub(super) fn reader(&self) -> Result<PhysicalRecordReader, CanonicalCommitReadFailure> {
        self.runtime
            .records()
            .map_err(CanonicalCommitReadFailure::Protection)
    }

    pub(super) fn head_anchor(&self, reader: &PhysicalRecordReader) -> Option<RecordAnchor> {
        reader.selected_record_anchors().get(self.name)
    }

    /// Finds `target` by descending jump pointers from the head.
    pub(super) fn locate(
        &self,
        reader: &PhysicalRecordReader,
        target: u64,
    ) -> Result<CanonicalCommitEntry, CanonicalCommitReadFailure> {
        let head = self.head_anchor(reader);
        let head_position = head.map_or(0, RecordAnchor::generation);
        let Some(head) = head.filter(|_| target >= 1 && target <= head_position) else {
            return Err(CanonicalCommitReadFailure::NotFound {
                head: head_position,
            });
        };
        if target == head_position {
            return self.entry_at(reader, head, target);
        }
        let mut header = read_header(reader, self, head.record())?;
        loop {
            let next = header
                .descend_toward(target)
                .ok_or(CanonicalCommitReadFailure::Damaged)?;
            let next_header = read_header(reader, self, next.record)?;
            if next_header.position != next.position {
                return Err(CanonicalCommitReadFailure::Damaged);
            }
            if next.position == target {
                return Ok(entry(next.record, &next_header));
            }
            header = next_header;
        }
    }

    fn entry_at(
        &self,
        reader: &PhysicalRecordReader,
        anchor: RecordAnchor,
        position: u64,
    ) -> Result<CanonicalCommitEntry, CanonicalCommitReadFailure> {
        let entry = self.entry_from(reader, anchor.record(), position)?;
        let bytes = read_record(reader, anchor.record())?;
        if <[u8; 32]>::from(Sha256::digest(&bytes)) != anchor.record_payload_sha256() {
            return Err(CanonicalCommitReadFailure::Damaged);
        }
        Ok(entry)
    }

    fn entry_from(
        &self,
        reader: &PhysicalRecordReader,
        record: PersistedRecordIdentity,
        position: u64,
    ) -> Result<CanonicalCommitEntry, CanonicalCommitReadFailure> {
        let header = read_header(reader, self, record)?;
        if header.position != position {
            return Err(CanonicalCommitReadFailure::Damaged);
        }
        Ok(entry(record, &header))
    }
}

fn entry(record: PersistedRecordIdentity, header: &JournalRecordHeader) -> CanonicalCommitEntry {
    CanonicalCommitEntry {
        position: header.position,
        record: PhysicalRecordId::from_persisted(record),
        payload_sha256: header.payload_sha256,
        payload_bytes: header.payload_bytes,
    }
}

fn persisted(record: PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
        .expect("a physical record id names a valid persisted identity")
}

/// Reads and validates only the header of a journal record.
pub(super) fn read_header(
    reader: &PhysicalRecordReader,
    journal: &CanonicalCommitJournal<'_>,
    record: PersistedRecordIdentity,
) -> Result<JournalRecordHeader, CanonicalCommitReadFailure> {
    let mut session = open(reader, record)?;
    let mut buffer = vec![0; MAXIMUM_HEADER_BYTES];
    let mut filled = 0;
    while filled < buffer.len() {
        let read = session
            .read_next(&mut buffer[filled..])
            .map_err(CanonicalCommitReadFailure::Stream)?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    JournalRecordHeader::decode(&buffer[..filled], journal.name, None)
        .map_err(|_| CanonicalCommitReadFailure::Damaged)
}

fn read_record(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<Vec<u8>, CanonicalCommitReadFailure> {
    let mut session = open(reader, record)?;
    let mut bytes = Vec::new();
    let mut chunk = vec![0; 64 * 1024];
    loop {
        let read = session
            .read_next(&mut chunk)
            .map_err(CanonicalCommitReadFailure::Stream)?;
        if read == 0 {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
}

fn open(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<crate::physical_runtime::RecordReadSession, CanonicalCommitReadFailure> {
    let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
    reader
        .open(PhysicalRecordId::from_persisted(record), limit)
        .map_err(CanonicalCommitReadFailure::Open)
}
