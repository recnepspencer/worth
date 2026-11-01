//! The original table and physical session, moved only after verified EOF.
use super::{AnchoredRecordMembership, AnchoredRecordReadFailure as Failure, AnchoredRecords};
use crate::physical_runtime::{
    PhysicalCheckpointReadProtectionDenial, PhysicalProtectedRootObservation, PhysicalRecordReader,
    RecordReadSession,
};

/// An actual streamed read's membership and original storage admission.
/// There is no public constructor, Clone, or owned-membership extraction.
/// Borrowing the existing membership creates no new publication authority.
/// The original session remains owned, avoiding a hidden registration-release
/// lock during conversion. Generic disposal is explicitly unbudgeted physical
/// session/table reclamation; retain the carrier through its consumers.
pub struct AnchoredRecordReadMembership {
    pub(super) membership: AnchoredRecordMembership,
    pub(super) _session: RecordReadSession,
    pub(super) root: PhysicalProtectedRootObservation,
    // Last: neither the original table nor session can outlive its admission.
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl AnchoredRecordReadMembership {
    pub fn membership(&self) -> &AnchoredRecordMembership {
        &self.membership
    }
}
impl AnchoredRecords<'_> {
    /// Loan the actual membership only after same-owner, exact-root and live
    /// protection checks. This performs no I/O, allocation or waiting. A later
    /// physical root never invalidates the still-protected original cut.
    pub fn validate_read_membership<'membership>(
        &self,
        reader: &PhysicalRecordReader,
        membership: &'membership AnchoredRecordReadMembership,
    ) -> Result<&'membership AnchoredRecordMembership, Failure> {
        if !reader.belongs_to(self.runtime) || reader.protected_root() != membership.root {
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
