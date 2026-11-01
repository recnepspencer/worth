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
}
impl AnchoredRecords<'_> {
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
            payload_sha256: stream.header.payload_sha256,
            payload_bytes: stream.header.payload_bytes,
        };
        Ok(Some(AnchoredRecordPayloadStream {
            stream,
            request: *request,
            observed,
            root: reader.protected_root(),
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
    /// Move the original verified table, its grant and its actual physical
    /// session without allocation, copying, re-reading or releasing a lease.
    /// Before verified EOF (including refusal), return this SAME stream.
    /// A completed fact wins over cancellation observed after its EOF.
    pub fn into_verified_membership(self) -> Result<super::AnchoredRecordReadMembership, Self> {
        if !self.stream.complete() {
            return Err(self);
        }
        Ok(self.stream.into_read_membership(self.observed, self.root))
    }

    /// Copy at most 64 KiB. `Ok(0)` means frame, payload and whole-record
    /// digests matched. Empty output cannot impersonate EOF. A refusal leaves
    /// this same stream non-advancing with its actual session custody retained.
    pub fn read_next(&mut self, output: &mut [u8]) -> Result<usize, Failure> {
        self.stream.read_next(Some(&self.request), output)
    }
}
