//! Why an active pointer gesture stops, read from what the host reported.

use worth_ui_host_contract::UiHostPointerCaptureEpoch;

use super::model::UiActivePointerGesture;
use crate::runtime::interaction::gesture::UiPointerGestureStopReason;
use crate::runtime::interaction::targeting::UiPointerGestureContinuityDenial;

pub(super) fn capture_change_reason(
    active: &UiActivePointerGesture,
    observed: UiHostPointerCaptureEpoch,
) -> Option<UiPointerGestureStopReason> {
    (active.capture_epoch != observed).then_some(UiPointerGestureStopReason::CaptureChanged {
        expected: active.capture_epoch,
        observed,
    })
}

pub(super) fn pointer_kind_change_reason(
    active: &UiActivePointerGesture,
    observed: crate::runtime::interaction::UiPrimaryPointerKind,
) -> Option<UiPointerGestureStopReason> {
    (active.kind != observed).then_some(UiPointerGestureStopReason::PointerDeviceKindChanged {
        expected: active.kind.host_kind(),
        observed: observed.host_kind(),
    })
}

pub(super) fn map_continuity_denial(
    denial: UiPointerGestureContinuityDenial,
) -> UiPointerGestureStopReason {
    match denial {
        UiPointerGestureContinuityDenial::PresentationDidNotAdvance => {
            UiPointerGestureStopReason::PresentationDidNotAdvance
        }
        UiPointerGestureContinuityDenial::SurfaceChanged => {
            UiPointerGestureStopReason::SurfaceChanged
        }
        UiPointerGestureContinuityDenial::BindingChanged => {
            UiPointerGestureStopReason::BindingChanged
        }
        UiPointerGestureContinuityDenial::MountedIncarnationChanged => {
            UiPointerGestureStopReason::MountedIncarnationChanged
        }
        UiPointerGestureContinuityDenial::TargetChangedWithinPresentation => {
            UiPointerGestureStopReason::TargetChangedWithinPresentation
        }
    }
}
