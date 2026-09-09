mod authority;
mod budget;
mod coordinator;
mod diagnostic_evidence;
mod evidence;
mod inspection_basis;
mod interaction_basis;
mod lease;
mod observation_basis;
mod rejection;
mod reservation;
mod snapshot;
mod successor_admission;
mod visual_capture_basis;
mod visual_lease;

pub(crate) use authority::UiMountedRetentionReservationIdentity;
pub(crate) use budget::DEFAULT_OBSERVATION_FRAME_CAPACITY;
pub use budget::{
    UiMountedFrameRetentionBudget, UiMountedFrameRetentionBudgetInput,
    UiMountedFrameRetentionDenial, UiMountedRetentionClass, UiMountedRetentionClassBudget,
};
pub(crate) use coordinator::UiMountedFrameRetentionCoordinator;
pub(crate) use coordinator::UiPresentedPointLookupDenial;
pub(crate) use diagnostic_evidence::UiRetainedMountedDiagnostics;
pub(crate) use evidence::{
    UiPresentedFrameBasisDenial, UiPresentedFrameBasisRelation, UiRetainedPresentedFrame,
};
pub(crate) use inspection_basis::{
    UiMountedDiagnosticInspectionBasis, UiMountedDiagnosticInspectionDenial,
    UiMountedFrameInspectionBasis, UiMountedFrameInspectionDenial,
    UiMountedFrameInspectionSelection, UiMountedFrameInspectionTarget,
};
#[cfg(test)]
pub(crate) use interaction_basis::motion_sampling_hit_test_mechanic_for_test;
pub(crate) use interaction_basis::{UiPresentedHitTestBasis, UiPresentedHitTestRow};
pub use lease::UiMountedRetentionLease;
pub(crate) use lease::{
    UiMountedDiagnosticRetentionLease, UiMountedObservationBasisLease, UiMountedVisualOverlayLease,
    UiMountedVisualSnapshotClass, UiMountedVisualSnapshotLease,
};
pub(crate) use lease::{UiMountedVisualLease, UiMountedVisualLeaseClass};
pub(crate) use observation_basis::UiMountedObservationBasisRetentionDenial;
pub use rejection::UiMountedFrameRetentionRejection;
pub(crate) use reservation::{
    UiMountedRetentionCommitDenial, UiMountedRetentionReservation, UiRetentionPreparedMountedFrame,
};
pub(crate) use snapshot::{UiMountedFrameRetentionSnapshot, UiMountedRetentionUsageSnapshot};
pub(crate) use visual_capture_basis::UiMountedVisualCaptureBasis;
pub(crate) use visual_lease::UiMountedVisualRetentionDenial;

mod hit_transition;
pub(crate) use hit_transition::UiCommittedPresentedHitTransition;
