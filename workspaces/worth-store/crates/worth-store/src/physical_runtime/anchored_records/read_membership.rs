//! One opaque shared allocation owns the original verified physical read.
use super::{AnchoredRecordMembership, AnchoredRecordReadFailure as Failure, AnchoredRecords};
use crate::physical_runtime::{
    PhysicalCheckpointReadProtectionDenial, PhysicalProtectedRootObservation, PhysicalRecordReader,
    RecordReadSession,
};
use std::sync::Arc;

mod retirement;
pub use retirement::{
    AnchoredRecordReadRetirement, AnchoredRecordReadRetirementDenial,
    AnchoredRecordReadRetirementProgress, RetiredAnchoredRecordReadMembership,
};

/// A verified streamed read with its ORIGINAL reader and intrinsic admissions.
/// Clone only shares the PRIVATE Arc; no Weak, Arc, unwrap, raw-reader getter,
/// public constructor or owned-membership extraction escapes. No physical
/// registration is added. Generic final disposal is explicitly unbudgeted;
/// retain this actual owner through all selected-section consumers.
#[derive(Clone)]
pub struct AnchoredRecordReadMembership {
    backing: Arc<ReadMembershipBacking>,
}
struct ReadMembershipBacking {
    membership: AnchoredRecordMembership,
    _session: RecordReadSession,
    reader: PhysicalRecordReader,
    root: PhysicalProtectedRootObservation,
    // Last: actual data and session/reader obligations retire before admission.
    _table_memory: Option<worth_execution::ExecutionMemoryReservation>,
    _shared_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl AnchoredRecordReadMembership {
    /// Consume this actual local share. Generic task disposal is unbudgeted;
    /// advance it to discharge physical obligations before retaining storage.
    pub fn into_retirement(self) -> AnchoredRecordReadRetirement {
        AnchoredRecordReadRetirement::new(self.backing)
    }

    pub fn membership(&self) -> &AnchoredRecordMembership {
        &self.backing.membership
    }
    pub(in crate::physical_runtime) fn original_reader(&self) -> &PhysicalRecordReader {
        &self.backing.reader
    }
    pub(super) fn reserve_shared_memory(
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Failure> {
        // Fixed concrete ArcInner layout using the existing checked header
        // convention; no guessed std collection node or caller size contract.
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ReadMembershipBacking>())
            .expect("fixed read membership shared layout fits")
            .0
            .pad_to_align();
        let bytes = u64::try_from(layout.size()).expect("fixed read membership layout fits u64");
        request
            .reserve_retained_memory(bytes)
            .map_err(Failure::Execution)
    }
    pub(super) fn from_verified(
        membership: AnchoredRecordMembership,
        session: RecordReadSession,
        reader: PhysicalRecordReader,
        root: PhysicalProtectedRootObservation,
        table_memory: Option<worth_execution::ExecutionMemoryReservation>,
        shared_memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        // The controlled open already reserved this exact allocation. EOF
        // conversion performs one honest allocation, with no request refresh.
        Self {
            backing: Arc::new(ReadMembershipBacking {
                membership,
                _session: session,
                reader,
                root,
                _table_memory: table_memory,
                _shared_memory: shared_memory,
            }),
        }
    }
}
impl AnchoredRecords<'_> {
    /// Loan actual membership after same-owner, original-reader, exact-root
    /// and live protection checks. No I/O, allocation or waiting. Newer physical
    /// roots do not invalidate this still-protected original read route.
    pub fn validate_read_membership<'membership>(
        &self,
        membership: &'membership AnchoredRecordReadMembership,
    ) -> Result<&'membership AnchoredRecordMembership, Failure> {
        let reader = membership.original_reader();
        if !reader.belongs_to(self.runtime) || reader.protected_root() != membership.backing.root {
            return Err(Failure::ForeignReader);
        }
        reader
            .try_require_live_protection()
            .map_err(|cause| match cause {
                PhysicalCheckpointReadProtectionDenial::Busy => Failure::ProtectionBusy,
                PhysicalCheckpointReadProtectionDenial::Protection(cause) => {
                    Failure::Protection(cause)
                }
                PhysicalCheckpointReadProtectionDenial::Request(cause) => {
                    Failure::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
                }
            })?;
        Ok(membership.membership())
    }
}
