//! Protected exact-length journal streaming without a payload allocation.
use super::frame::{JournalRecordHeader, MAXIMUM_HEADER_BYTES};
use super::{OrderedJournal, OrderedJournalEntry, OrderedJournalReadFailure as Failure};
use crate::physical_runtime::{RecordByteLimit, RecordReadLimits, RecordReadSession};
use sha2::{Digest, Sha256};

/// A controlled stream borrows the SAME original request. Entry coordinates
/// are descriptive until verified EOF. Historical linkage relies on the
/// protected physical record identities and frame integrity: journal jump
/// pointers do not form a transitive cryptographic hash chain.
pub struct OrderedJournalPayloadStream<'request, 'authority> {
    request: worth_execution::ExecutionRequest<'request, 'authority>,
    refused: bool,
    entry: OrderedJournalEntry,
    session: RecordReadSession,
    remaining: u64,
    payload_digest: Sha256,
    record_digest: Sha256,
    expected_record_digest: Option<[u8; 32]>,
    complete: bool,
}

impl OrderedJournal<'_> {
    /// Resolve exact length before a caller allocates payload storage. Each
    /// read uses caller-owned scratch; zero is returned only after verification.
    pub fn open_payload<'request, 'authority>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        position: u64,
        maximum_payload_bytes: u64,
    ) -> Result<OrderedJournalPayloadStream<'request, 'authority>, Failure> {
        let reader = self.reader()?;
        self.open_payload_from(&reader, execution, position, maximum_payload_bytes)
    }

    pub fn head_position_from(
        &self,
        reader: &crate::physical_runtime::PhysicalRecordReader,
    ) -> Result<u64, Failure> {
        if !reader.belongs_to(self.runtime) {
            return Err(Failure::ForeignReader);
        }
        Ok(self
            .head_anchor(reader)
            .map_or(0, |anchor| anchor.generation()))
    }

    /// Use this original protected reader throughout a frozen owner cut.
    pub fn open_payload_from<'request, 'authority>(
        &self,
        reader: &crate::physical_runtime::PhysicalRecordReader,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        position: u64,
        maximum_payload_bytes: u64,
    ) -> Result<OrderedJournalPayloadStream<'request, 'authority>, Failure> {
        if !reader.belongs_to(self.runtime) {
            return Err(Failure::ForeignReader);
        }
        let entry = self.locate_header(reader, position, Some(execution))?;
        if entry.payload_bytes() > maximum_payload_bytes {
            return Err(Failure::LimitExceeded {
                payload_bytes: entry.payload_bytes(),
            });
        }
        let header = super::lookup::read_header_controlled(
            reader,
            self,
            super::lookup::persisted(entry.record()),
            Some(execution),
        )?;
        let limits = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
        let mut session = reader.open(entry.record(), limits).map_err(Failure::Open)?;
        let mut scratch = [0; MAXIMUM_HEADER_BYTES];
        let bytes = &mut scratch[..header.encoded_bytes()];
        let mut filled = 0;
        while filled < bytes.len() {
            execution.check_live().map_err(|stop| {
                Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
            })?;
            let count = session
                .read_next(&mut bytes[filled..])
                .map_err(Failure::Stream)?;
            if count == 0 {
                return Err(Failure::Damaged);
            }
            filled += count;
        }
        let actual = JournalRecordHeader::decode_funded(
            bytes,
            self.name,
            session.observation().bytes_requested(),
            execution,
        )?;
        if actual != header
            || header.position != position
            || header.payload_bytes != entry.payload_bytes()
            || *header.payload_sha256.bytes() != entry.payload_sha256()
        {
            return Err(Failure::Damaged);
        }
        let expected_record_digest = self
            .head_anchor(reader)
            .filter(|anchor| anchor.generation() == position)
            .map(|anchor| anchor.record_payload_sha256());
        Ok(OrderedJournalPayloadStream {
            request: *execution,
            refused: false,
            entry,
            session,
            remaining: entry.payload_bytes(),
            payload_digest: Sha256::new(),
            record_digest: Sha256::new_with_prefix(bytes),
            expected_record_digest,
            complete: false,
        })
    }
}

impl OrderedJournalPayloadStream<'_, '_> {
    /// Declared target coordinates, not a verified-EOF receipt.
    pub fn entry(&self) -> OrderedJournalEntry {
        self.entry
    }

    pub fn read_next(&mut self, buffer: &mut [u8]) -> Result<usize, Failure> {
        if self.complete {
            return Ok(0);
        }
        if self.refused {
            return Err(Failure::PreviouslyRefused);
        }
        let result = self.read_live(buffer);
        self.refused = result.is_err();
        result
    }

    fn read_live(&mut self, buffer: &mut [u8]) -> Result<usize, Failure> {
        self.request.check_live().map_err(|stop| {
            Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
        })?;
        if self.remaining == 0 {
            if self
                .session
                .read_next(&mut [0; 1])
                .map_err(Failure::Stream)?
                != 0
                || <[u8; 32]>::from(self.payload_digest.clone().finalize())
                    != self.entry.payload_sha256()
                || self.expected_record_digest.is_some_and(|expected| {
                    <[u8; 32]>::from(self.record_digest.clone().finalize()) != expected
                })
            {
                return Err(Failure::Damaged);
            }
            self.complete = true;
            return Ok(0);
        }
        if buffer.is_empty() {
            return Err(Failure::EmptyReadBuffer);
        }
        let bound = usize::try_from(self.remaining)
            .unwrap_or(usize::MAX)
            .min(buffer.len())
            .min(64 * 1024);
        let count = self
            .session
            .read_next(&mut buffer[..bound])
            .map_err(Failure::Stream)?;
        if count == 0 {
            return Err(Failure::Damaged);
        }
        self.remaining -= count as u64;
        self.payload_digest.update(&buffer[..count]);
        self.record_digest.update(&buffer[..count]);
        Ok(count)
    }
}
