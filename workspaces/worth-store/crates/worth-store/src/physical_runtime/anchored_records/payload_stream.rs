//! Logical owner payload from one exact protected-reader anchor.
use super::verified_value::AnchoredValueStream;
use super::{
    AnchoredRecordName, AnchoredRecordReadFailure as Failure, AnchoredRecordVersion,
    AnchoredRecords,
};
use crate::physical_runtime::PhysicalRecordReader;
use worth_execution::ExecutionRequest;

/// The header's coordinates remain descriptive until verified EOF. This type
/// exposes no member-list before verified EOF. The consuming verified-membership
/// handoff moves its original table and session without copying. It borrows the
/// SAME original request, never a replacement grant. Generic disposal follows
/// the physical session's ordinary, unbudgeted protection-release contract.
pub struct AnchoredRecordPayloadStream<'request, 'authority> {
    stream: AnchoredValueStream,
    request: ExecutionRequest<'request, 'authority>,
    observed: AnchoredRecordVersion,
    root: crate::physical_runtime::PhysicalProtectedRootObservation,
    shared_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl AnchoredRecords<'_> {
    /// Open another anchor under this verified Home read's ORIGINAL reader.
    /// No latest-root acquisition or raw reader extraction occurs.
    pub fn open_payload_from_read<'request, 'authority>(
        &self,
        read: &super::AnchoredRecordReadMembership,
        request: &ExecutionRequest<'request, 'authority>,
        name: AnchoredRecordName,
        maximum_payload_bytes: u64,
    ) -> Result<Option<AnchoredRecordPayloadStream<'request, 'authority>>, Failure> {
        self.validate_read_membership(read)?;
        self.open_payload_from(read.original_reader(), request, name, maximum_payload_bytes)
    }
    /// Open the exact anchor selected by this caller's reader. Header lengths
    /// agree with the physical record before any table/payload allocation.
    /// Perform this I/O outside logical reservations. Payload copying must use
    /// its original request's actual allocation owner.
    pub fn open_payload_from<'request, 'authority>(
        &self,
        reader: &PhysicalRecordReader,
        request: &ExecutionRequest<'request, 'authority>,
        name: AnchoredRecordName,
        maximum_payload_bytes: u64,
    ) -> Result<Option<AnchoredRecordPayloadStream<'request, 'authority>>, Failure> {
        if !reader.belongs_to(self.runtime) {
            return Err(Failure::ForeignReader);
        }
        request.check_live().map_err(|cause| {
            Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
        })?;
        let Some(anchor) = self.current_anchor(reader, name) else {
            return Ok(None);
        };
        // Reserve the fixed future shared carrier BEFORE record I/O and table
        // growth, including when this caller ultimately retains only payload.
        let shared_memory = super::AnchoredRecordReadMembership::reserve_shared_memory(request)?;
        let stream = AnchoredValueStream::open(
            reader,
            name,
            anchor,
            Some(maximum_payload_bytes),
            Some(request),
        )?;
        let observed = AnchoredRecordVersion {
            name,
            generation: anchor.generation(),
            record: crate::physical_runtime::PhysicalRecordId::from_persisted(anchor.record()),
            payload_sha256: *stream.header.payload_sha256.bytes(),
            payload_bytes: stream.header.payload_bytes,
        };
        Ok(Some(AnchoredRecordPayloadStream {
            stream,
            request: *request,
            observed,
            root: reader.protected_root(),
            shared_memory,
        }))
    }
}
impl AnchoredRecordPayloadStream<'_, '_> {
    /// Declared size, bounded by the actual physical record and caller limit.
    pub fn payload_bytes(&self) -> u64 {
        self.observed.payload_bytes()
    }
    /// Original selected generation; this alone claims no payload integrity.
    pub fn observed_generation(&self) -> u64 {
        self.observed.generation()
    }
    /// Only actual complete EOF makes this validated description available.
    pub fn verified_version(&self) -> Option<AnchoredRecordVersion> {
        self.stream.complete().then_some(self.observed)
    }
    /// Move the original verified table, its grants, session and reader into
    /// ONE pre-funded private Arc allocation; no copying, re-reading or lease
    /// release. There is no fresh request consultation after actual EOF.
    /// Before verified EOF (including refusal), or for a mismatching reader,
    /// return BOTH the SAME stream and supplied reader. No registration is
    /// created or released by this conversion.
    /// A completed fact wins over cancellation observed after its EOF.
    pub fn into_verified_membership(
        self,
        reader: PhysicalRecordReader,
    ) -> Result<super::AnchoredRecordReadMembership, (Self, PhysicalRecordReader)> {
        if !self.stream.complete() || reader.protected_root() != self.root {
            return Err((self, reader));
        }
        Ok(self
            .stream
            .into_read_membership(self.observed, self.root, reader, self.shared_memory))
    }

    /// Copy at most 64 KiB. `Ok(0)` means frame, payload and whole-record
    /// digests matched. Empty output cannot impersonate EOF. A refusal leaves
    /// this same stream non-advancing with its actual session custody retained.
    pub fn read_next(&mut self, output: &mut [u8]) -> Result<usize, Failure> {
        self.stream.read_next(Some(&self.request), output)
    }
}
