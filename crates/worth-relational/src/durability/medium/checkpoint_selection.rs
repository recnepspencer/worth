//! Read-only selection questions sent only through the already attached medium.
use super::RelationalDurableMemberLocator;
use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

/// An original runtime capture. Untrusted byte constructors cannot mint this
/// subject, and copied bytes cannot substitute for its process-local identity.
#[derive(Debug, Clone)]
pub struct OpaqueCapturedCheckpointSubject {
    identity: Arc<()>,
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    required: Arc<[RelationalDurableMemberLocator]>,
}

impl OpaqueCapturedCheckpointSubject {
    pub(crate) fn new(
        bytes: ExecutionImmutableBytes,
        region: std::ops::Range<usize>,
        required: Arc<[RelationalDurableMemberLocator]>,
    ) -> Self {
        Self {
            identity: Arc::new(()),
            bytes,
            region,
            required,
        }
    }
    pub fn same_capture(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }
    pub fn required_members(&self) -> &[RelationalDurableMemberLocator] {
        &self.required
    }
}

/// Descriptive actual-selection identity supplied by the trusted medium.
/// This is not a durability witness and opens no live cutover door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointSelectionIdentity {
    pub generation: u64,
    pub manifest_sha256: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointSelectionObservation {
    NotSelected,
    Unknown,
    Selected(CheckpointSelectionIdentity),
}

#[derive(Debug)]
pub enum RelationalCheckpointAcknowledgementDenial {
    CaptureNotBound,
    ForeignSource,
    StaleCapture,
    SelectionNotKnown,
    Storage(crate::storage::data::RelationalStorageUnavailableCause),
    Stopped(worth_execution::MapKernelStop),
}
