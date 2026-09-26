use std::marker::PhantomData;
use worth_ui_inspection::UiVisualArtifactPolicy;

use super::capture_progression::{UiPinnedVisualCapture, UiRequestedHostVisualCapture};

pub struct UiPendingVisualCapture<Target, ArtifactPolicy: UiVisualArtifactPolicy> {
    phase: UiPendingVisualCapturePhase<Target, ArtifactPolicy>,
}

enum UiPendingVisualCapturePhase<Target, Policy: UiVisualArtifactPolicy> {
    Pinned(UiPinnedVisualCapture<Target, Policy>),
    HostRequested(UiRequestedHostVisualCapture<Target, Policy>),
    DerivedRegion(UiPendingDerivedRegionCapture<Target, Policy>),
}

pub(crate) struct UiPendingDerivedRegionCapture<Target, Policy: UiVisualArtifactPolicy> {
    pub(crate) capture_identity: u64,
    pub(crate) deadline: Option<worth_ui_inspection::UiVisualCaptureDeadline>,
    pub(crate) source: super::UiRetainedVisualSnapshotSource,
    pub(crate) region: worth_ui_inspection::UiClientPhysicalRect,
    _target: PhantomData<fn() -> Target>,
    _policy: PhantomData<Policy>,
}

pub(crate) struct UiPendingDerivedRegionInput {
    pub(crate) capture_identity: u64,
    pub(crate) deadline: Option<worth_ui_inspection::UiVisualCaptureDeadline>,
    pub(crate) source: super::UiRetainedVisualSnapshotSource,
    pub(crate) region: worth_ui_inspection::UiClientPhysicalRect,
}

pub(crate) enum UiPendingVisualCaptureRoute<Target, Policy: UiVisualArtifactPolicy> {
    Host(UiPendingVisualCapture<Target, Policy>),
    DerivedRegion(UiPendingDerivedRegionCapture<Target, Policy>),
}

#[must_use = "capture polling returns a successor handle or terminal outcome that must be handled"]
pub enum UiVisualCapturePoll<Target, ArtifactPolicy: UiVisualArtifactPolicy> {
    Pending(UiPendingVisualCapture<Target, ArtifactPolicy>),
    Completed(UiVisualSnapshotOutcome<ArtifactPolicy::CapturedPosture>),
}

#[must_use = "visual snapshot outcomes must be handled"]
pub enum UiVisualSnapshotOutcome<ArtifactPosture: UiVisualArtifactPolicy> {
    Captured(super::UiVisualSnapshotReceipt<ArtifactPosture>),
    Superseded(worth_ui_inspection::UiVisualSnapshotSuperseded),
    Omitted(worth_ui_inspection::UiVisualSnapshotOmission),
    Denied(worth_ui_inspection::UiVisualSnapshotDenial),
    Indeterminate(worth_ui_inspection::UiVisualSnapshotIndeterminate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiVisualCancellationReceipt {
    capture_identity: u64,
    posture: UiVisualCancellationPosture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualCancellationPosture {
    BeforeHostRequest,
    CancelledBeforeReadback,
    ReadbackMayHaveBegun,
    CleanupIndeterminate,
}

impl<Target, Policy> UiPendingVisualCapture<Target, Policy>
where
    Policy: UiVisualArtifactPolicy,
{
    pub(crate) fn pinned(capture: UiPinnedVisualCapture<Target, Policy>) -> Self {
        Self {
            phase: UiPendingVisualCapturePhase::Pinned(capture),
        }
    }

    pub(crate) fn host_requested(capture: UiRequestedHostVisualCapture<Target, Policy>) -> Self {
        Self {
            phase: UiPendingVisualCapturePhase::HostRequested(capture),
        }
    }

    pub(crate) fn derived_region(input: UiPendingDerivedRegionInput) -> Self {
        Self {
            phase: UiPendingVisualCapturePhase::DerivedRegion(UiPendingDerivedRegionCapture {
                capture_identity: input.capture_identity,
                deadline: input.deadline,
                source: input.source,
                region: input.region,
                _target: PhantomData,
                _policy: PhantomData,
            }),
        }
    }

    pub fn capture_identity(&self) -> u64 {
        match &self.phase {
            UiPendingVisualCapturePhase::Pinned(capture) => capture.capture_identity(),
            UiPendingVisualCapturePhase::HostRequested(capture) => capture.capture_identity(),
            UiPendingVisualCapturePhase::DerivedRegion(capture) => capture.capture_identity,
        }
    }

    pub(crate) const fn host_request(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostVisualCaptureRequest> {
        match &self.phase {
            UiPendingVisualCapturePhase::Pinned(_) => None,
            UiPendingVisualCapturePhase::HostRequested(capture) => Some(capture.host_request()),
            UiPendingVisualCapturePhase::DerivedRegion(_) => None,
        }
    }

    pub(crate) const fn capture_deadline(
        &self,
    ) -> Option<worth_ui_inspection::UiVisualCaptureDeadline> {
        match &self.phase {
            UiPendingVisualCapturePhase::Pinned(capture) => capture.capture_deadline(),
            UiPendingVisualCapturePhase::HostRequested(capture) => capture.capture_deadline(),
            UiPendingVisualCapturePhase::DerivedRegion(capture) => capture.deadline,
        }
    }

    pub(crate) fn into_route(self) -> UiPendingVisualCaptureRoute<Target, Policy> {
        match self.phase {
            UiPendingVisualCapturePhase::DerivedRegion(capture) => {
                UiPendingVisualCaptureRoute::DerivedRegion(capture)
            }
            phase @ (UiPendingVisualCapturePhase::Pinned(_)
            | UiPendingVisualCapturePhase::HostRequested(_)) => {
                UiPendingVisualCaptureRoute::Host(Self { phase })
            }
        }
    }

    pub(crate) fn into_host_requested<Omission>(
        self,
        build_request: impl FnOnce(
            &UiPinnedVisualCapture<Target, Policy>,
        ) -> Result<
            worth_ui_host_contract::UiHostVisualCaptureRequest,
            Omission,
        >,
    ) -> Result<UiRequestedHostVisualCapture<Target, Policy>, Omission> {
        match self.phase {
            UiPendingVisualCapturePhase::Pinned(capture) => {
                let request = build_request(&capture)?;
                Ok(capture.request_host(request))
            }
            UiPendingVisualCapturePhase::HostRequested(capture) => Ok(capture),
            UiPendingVisualCapturePhase::DerivedRegion(_) => {
                unreachable!("derived captures never enter the host-request progression")
            }
        }
    }

    pub(crate) fn cancel_before_host(self) -> UiVisualCancellationReceipt {
        debug_assert!(matches!(
            self.phase,
            UiPendingVisualCapturePhase::Pinned(_) | UiPendingVisualCapturePhase::DerivedRegion(_)
        ));
        UiVisualCancellationReceipt {
            capture_identity: self.capture_identity(),
            posture: UiVisualCancellationPosture::BeforeHostRequest,
        }
    }

    pub(crate) fn cancel(
        self,
        posture: UiVisualCancellationPosture,
    ) -> UiVisualCancellationReceipt {
        UiVisualCancellationReceipt {
            capture_identity: self.capture_identity(),
            posture,
        }
    }
}

impl UiVisualCancellationReceipt {
    pub const fn capture_identity(self) -> u64 {
        self.capture_identity
    }

    pub const fn posture(self) -> UiVisualCancellationPosture {
        self.posture
    }

    pub const fn host_readback_began(self) -> Option<bool> {
        match self.posture {
            UiVisualCancellationPosture::BeforeHostRequest
            | UiVisualCancellationPosture::CancelledBeforeReadback => Some(false),
            UiVisualCancellationPosture::ReadbackMayHaveBegun => Some(true),
            UiVisualCancellationPosture::CleanupIndeterminate => None,
        }
    }
}
