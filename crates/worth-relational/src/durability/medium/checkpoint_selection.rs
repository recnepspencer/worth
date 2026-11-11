//! Read-only selection questions sent only through the already attached medium.
use super::RelationalDurableMemberLocator;
use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

/// An original runtime capture. Untrusted byte constructors cannot mint this
/// subject, and copied bytes cannot substitute for its process-local identity.
#[derive(Debug, Clone)]
pub struct OpaqueCapturedCheckpointSubject {
    identity: Arc<CapturedMembers>,
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
}

#[derive(Debug)]
struct CapturedMembers {
    initial_inputs:
        Option<crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact>,
    required: worth_execution::ExecutionArray<RelationalDurableMemberLocator>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl OpaqueCapturedCheckpointSubject {
    pub(crate) fn new(
        bytes: ExecutionImmutableBytes,
        region: std::ops::Range<usize>,
        members: &[crate::durability::data::RelationalCheckpointMember],
        initial_inputs: Option<
            crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact,
        >,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::RelationalNativeCheckpointCaptureDenial> {
        use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
        let size = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<CapturedMembers>())
            .map_err(|_| {
                Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(size as u64)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut required =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, members.len())
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        for member in members {
            required
                .push(member.locator())
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
        let required = required
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        Ok(Self {
            identity: Arc::new(CapturedMembers {
                required,
                initial_inputs,
                _memory: memory,
            }),
            bytes,
            region,
        })
    }
    /// Descriptive original input coordinate retained by this exact capture.
    pub fn initial_inputs_coordinate(&self) -> Option<(u64, [u8; 32])> {
        self.identity
            .initial_inputs
            .as_ref()
            .map(|image| (image.revision(), image.payload_digest()))
    }
    pub fn same_capture(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }
    pub fn required_members(&self) -> &[RelationalDurableMemberLocator] {
        self.identity.required.elements()
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

/// Historical acknowledgement progress of one original capture.
/// Selection validity continues to be decided by the attached owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCheckpointAcknowledgementProgress {
    AwaitingSelectionObservation,
    PlacingData,
    RetiringSources,
    Complete,
}

#[derive(Debug)]
pub enum RelationalCheckpointAcknowledgementDenial {
    CaptureNotBound,
    ForeignSource,
    StaleCapture,
    SelectionNotKnown,
    Storage(crate::storage::data::RelationalStorageUnavailableCause),
    Stopped(worth_execution::MapKernelStop),
    Busy,
    Recovery(crate::durability::data::RelationalRecoveryStop),
    OwnedRead(super::RelationalDurableOwnedReadRefusal),
    Retirement(super::RelationalDurableMemberRetirementRefusal),
}
