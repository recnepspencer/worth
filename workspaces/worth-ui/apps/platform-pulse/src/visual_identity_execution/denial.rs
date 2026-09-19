use worth_ui::facade::app::UiMountedInspectionOmission;
use worth_ui::facade::inspection::{UiVisualOverlayDenial, UiVisualSnapshotDenial};

use crate::lifecycle_observation_publication::PlatformPulseObservationPublicationDenial;

#[derive(Debug)]
pub(crate) enum PlatformPulseVisualExecutionDenial {
    ReentrantTransition,
    InitialFrameAlreadyArmed,
    ClockOverflow,
    TickExhausted,
    CaptureMountedFrameUnavailable(UiMountedInspectionOmission),
    ComparisonMountedFrameUnavailable(UiMountedInspectionOmission),
    MountedFrameReadinessUnavailable(UiMountedInspectionOmission),
    MountedVisualTarget,
    SnapshotAdmission(UiVisualSnapshotDenial),
    SnapshotDeadline(PlatformPulseVisualCapturePhase),
    SnapshotOmitted,
    SnapshotDenied(UiVisualSnapshotDenial),
    SnapshotIndeterminate,
    PointCoordinate,
    PointOmitted,
    PointUnsupported,
    PointIdentityMismatch,
    AuthoredNameMismatch,
    OverlayTarget(UiVisualOverlayDenial),
    OverlayAdmission(UiVisualOverlayDenial),
    OverlayPublication(UiVisualOverlayDenial),
    OverlayClear(UiVisualOverlayDenial),
    ReplacementBeforeOverlayClear,
    SnapshotRelation(worth_ui::facade::inspection::UiVisualSnapshotRelationDenial),
    SnapshotStillCurrent,
    SnapshotSuccessorMismatch,
    ShutdownNotQuiescent,
    ComparisonOmitted(worth_ui::facade::inspection::UiVisualSnapshotComparisonOmission),
    ComparisonExpired(worth_ui::facade::inspection::UiVisualSnapshotComparisonExpiry),
    ComparisonIncompatible(worth_ui::facade::inspection::UiVisualSnapshotComparisonIncompatibility),
    ComparisonDenied(worth_ui::facade::inspection::UiVisualSnapshotComparisonDenial),
    Observation(PlatformPulseObservationPublicationDenial),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PlatformPulseVisualCapturePhase {
    Initial,
    InitialReadiness,
    Rebase,
    RebaseReadiness,
    Refresh,
    RefreshReadiness,
    Comparison,
    ComparisonReadiness,
}

impl std::fmt::Display for PlatformPulseVisualExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComparisonMountedFrameUnavailable(omission) => {
                write!(
                    formatter,
                    "comparison mounted frame unavailable: {omission:?}"
                )
            }
            Self::MountedFrameReadinessUnavailable(omission) => {
                write!(
                    formatter,
                    "mounted-frame readiness unavailable: {omission:?}"
                )
            }
            Self::SnapshotAdmission(denial) => {
                write!(formatter, "snapshot admission: {denial:?}")
            }
            Self::SnapshotDenied(denial) => write!(formatter, "snapshot denied: {denial:?}"),
            Self::SnapshotDeadline(phase) => {
                write!(formatter, "snapshot deadline during {phase:?}")
            }
            Self::ComparisonOmitted(omission) => {
                write!(formatter, "comparison omitted: {omission:?}")
            }
            Self::ComparisonExpired(expiry) => write!(formatter, "comparison expired: {expiry:?}"),
            Self::ComparisonIncompatible(incompatibility) => {
                write!(formatter, "comparison incompatible: {incompatibility:?}")
            }
            Self::ComparisonDenied(denial) => write!(formatter, "comparison denied: {denial:?}"),
            Self::OverlayTarget(denial) => write!(formatter, "overlay target: {denial:?}"),
            Self::OverlayAdmission(denial) => write!(formatter, "overlay admission: {denial:?}"),
            Self::OverlayPublication(denial) => {
                write!(formatter, "overlay publication: {denial:?}")
            }
            Self::OverlayClear(denial) => write!(formatter, "overlay clear: {denial:?}"),
            Self::SnapshotRelation(denial) => write!(formatter, "snapshot relation: {denial:?}"),
            Self::Observation(denial) => write!(formatter, "observation publication: {denial:?}"),
            denial => write!(formatter, "{denial:?}"),
        }
    }
}
