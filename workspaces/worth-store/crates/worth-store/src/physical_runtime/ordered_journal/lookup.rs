use sha2::{Digest, Sha256};
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchor};

use super::frame::{JournalRecordHeader, MAXIMUM_HEADER_BYTES};
use super::{
    OrderedJournal, OrderedJournalEntry, OrderedJournalPayload, OrderedJournalReadFailure,
    OrderedJournalResolution,
};
use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

impl OrderedJournal<'_> {
    /// Describe one exact position from this original protected reader.
    /// Reader affinity is checked before I/O; the original request is checked
    /// even when the position is absent. This funded header lookup does not
    /// read/verify the complete payload and issues no verified-byte capability.
    /// Physical identities and frames remain bound to the reader's selected root.
    pub fn describe_entry_from(
        &self,
        reader: &PhysicalRecordReader,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        if !reader.belongs_to(self.runtime) {
            return Err(OrderedJournalReadFailure::ForeignReader);
        }
        request.check_live().map_err(|stop| {
            OrderedJournalReadFailure::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
        })?;
        self.locate_header(reader, position, Some(request))
    }

    /// The newest durable entry, or `None` for an empty journal.
    pub fn head(&self) -> Result<Option<OrderedJournalEntry>, OrderedJournalReadFailure> {
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
    ) -> Result<OrderedJournalResolution, OrderedJournalReadFailure> {
        let unresolved = self
            .runtime
            .ordered_journal_state()
            .unresolved
            .get(&self.name)
            .copied();
        let reader = self.reader()?;
        let head = self
            .head_anchor(&reader)
            .map_or(0, RecordAnchor::generation);
        if position == 0 || position > head {
            return Ok(if unresolved == Some(position) {
                OrderedJournalResolution::Unresolved
            } else {
                OrderedJournalResolution::NotCommitted { head }
            });
        }
        let entry = self.locate(&reader, position)?;
        Ok(if entry.payload_sha256 == payload_sha256 {
            OrderedJournalResolution::Committed(entry)
        } else {
            OrderedJournalResolution::Conflict(entry)
        })
    }

    /// Reads one entry's payload, verified against its recorded digest.
    pub fn read(
        &self,
        position: u64,
        maximum_payload_bytes: u64,
    ) -> Result<OrderedJournalPayload, OrderedJournalReadFailure> {
        let reader = self.reader()?;
        let entry = self.locate(&reader, position)?;
        if entry.payload_bytes > maximum_payload_bytes {
            return Err(OrderedJournalReadFailure::LimitExceeded {
                payload_bytes: entry.payload_bytes,
            });
        }
        let limit = MAXIMUM_HEADER_BYTES as u64 + entry.payload_bytes;
        let bytes = read_record(&reader, persisted(entry.record), limit)?;
        let header = JournalRecordHeader::decode(&bytes, self.name, Some(bytes.len() as u64))
            .map_err(|_| OrderedJournalReadFailure::Damaged)?;
        let payload = bytes[header.encoded_bytes()..].to_vec();
        if header.position != position || header.verify_payload(&payload).is_err() {
            return Err(OrderedJournalReadFailure::Damaged);
        }
        Ok(OrderedJournalPayload {
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
    ) -> Result<Vec<OrderedJournalEntry>, OrderedJournalReadFailure> {
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
                .ok_or(OrderedJournalReadFailure::Damaged)?;
            entry = self.entry_from(&reader, previous.record, previous.position)?;
        }
    }

    pub(super) fn reader(&self) -> Result<PhysicalRecordReader, OrderedJournalReadFailure> {
        self.runtime
            .records()
            .map_err(OrderedJournalReadFailure::Protection)
    }

    pub(super) fn head_anchor(&self, reader: &PhysicalRecordReader) -> Option<RecordAnchor> {
        reader.selected_record_anchors().get(self.name.anchor())
    }

    /// Finds `target` by descending jump pointers from the head.
    pub(super) fn locate(
        &self,
        reader: &PhysicalRecordReader,
        target: u64,
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        if let Some(head) = self
            .head_anchor(reader)
            .filter(|head| head.generation() == target)
        {
            return self.entry_at(reader, head, target);
        }
        self.locate_header(reader, target, None)
    }

    /// Header-only resolution for the protected streaming owner. The selected
    /// root protects immutable record identities and their physical integrity.
    /// The payload stream verifies the target payload, and the full selecting
    /// anchor digest when the target is the head. Spine entries carry physical
    /// identities, not predecessor digests.
    pub(super) fn locate_header(
        &self,
        reader: &PhysicalRecordReader,
        target: u64,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        let head = self.head_anchor(reader);
        let head_position = head.map_or(0, RecordAnchor::generation);
        let Some(head) = head.filter(|_| target >= 1 && target <= head_position) else {
            return Err(OrderedJournalReadFailure::NotFound {
                head: head_position,
            });
        };
        if target == head_position {
            let header = read_header_controlled(reader, self, head.record(), execution)?;
            if header.position != target {
                return Err(OrderedJournalReadFailure::Damaged);
            }
            return Ok(entry(head.record(), &header));
        }
        let mut header = read_header_controlled(reader, self, head.record(), execution)?;
        loop {
            let next = header
                .descend_toward(target)
                .ok_or(OrderedJournalReadFailure::Damaged)?;
            let next_header = read_header_controlled(reader, self, next.record, execution)?;
            if next_header.position != next.position {
                return Err(OrderedJournalReadFailure::Damaged);
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
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        let entry = self.entry_from(reader, anchor.record(), position)?;
        let limit = MAXIMUM_HEADER_BYTES as u64 + entry.payload_bytes;
        let bytes = read_record(reader, anchor.record(), limit)?;
        if <[u8; 32]>::from(Sha256::digest(&bytes)) != anchor.record_payload_sha256() {
            return Err(OrderedJournalReadFailure::Damaged);
        }
        Ok(entry)
    }

    fn entry_from(
        &self,
        reader: &PhysicalRecordReader,
        record: PersistedRecordIdentity,
        position: u64,
    ) -> Result<OrderedJournalEntry, OrderedJournalReadFailure> {
        let header = read_header(reader, self, record)?;
        if header.position != position {
            return Err(OrderedJournalReadFailure::Damaged);
        }
        Ok(entry(record, &header))
    }
}

fn entry(record: PersistedRecordIdentity, header: &JournalRecordHeader) -> OrderedJournalEntry {
    OrderedJournalEntry {
        position: header.position,
        record: PhysicalRecordId::from_persisted(record),
        payload_sha256: header.payload_sha256,
        payload_bytes: header.payload_bytes,
    }
}

pub(super) fn persisted(record: PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
        .expect("a physical record id names a valid persisted identity")
}

/// Reads and validates only the header of a journal record.
pub(super) fn read_header(
    reader: &PhysicalRecordReader,
    journal: &OrderedJournal<'_>,
    record: PersistedRecordIdentity,
) -> Result<JournalRecordHeader, OrderedJournalReadFailure> {
    read_header_controlled(reader, journal, record, None)
}

pub(super) fn read_header_controlled(
    reader: &PhysicalRecordReader,
    journal: &OrderedJournal<'_>,
    record: PersistedRecordIdentity,
    execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<JournalRecordHeader, OrderedJournalReadFailure> {
    let mut session = open(reader, record)?;
    let mut buffer = [0; MAXIMUM_HEADER_BYTES];
    let mut filled = 0;
    while filled < buffer.len() {
        if let Some(execution) = execution {
            execution.check_live().map_err(|stop| {
                OrderedJournalReadFailure::Execution(worth_execution::WorkCeilingDenial::Stopped(
                    stop,
                ))
            })?;
        }
        let read = session
            .read_next(&mut buffer[filled..])
            .map_err(OrderedJournalReadFailure::Stream)?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    match execution {
        Some(execution) => JournalRecordHeader::decode_funded(
            &buffer[..filled],
            journal.name,
            session.observation().bytes_requested(),
            execution,
        ),
        None => JournalRecordHeader::decode(&buffer[..filled], journal.name, None)
            .map_err(|_| OrderedJournalReadFailure::Damaged),
    }
}

/// Reads a whole record, refusing one longer than its header declared.
fn read_record(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    maximum_bytes: u64,
) -> Result<Vec<u8>, OrderedJournalReadFailure> {
    let mut session = open(reader, record)?;
    let mut bytes = Vec::new();
    let mut chunk = vec![0; 64 * 1024];
    loop {
        let read = session
            .read_next(&mut chunk)
            .map_err(OrderedJournalReadFailure::Stream)?;
        if read == 0 {
            return Ok(bytes);
        }
        if bytes.len() as u64 + read as u64 > maximum_bytes {
            return Err(OrderedJournalReadFailure::Damaged);
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
}

fn open(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<crate::physical_runtime::RecordReadSession, OrderedJournalReadFailure> {
    let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
    reader
        .open(PhysicalRecordId::from_persisted(record), limit)
        .map_err(OrderedJournalReadFailure::Open)
}
