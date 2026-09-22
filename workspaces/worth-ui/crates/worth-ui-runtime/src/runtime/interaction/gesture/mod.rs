mod pointer;
mod stop;

pub use super::targeting::UiPointerGestureContinuityKind;
pub use pointer::{
    UiPointerGesturePressReceipt, UiTargetedPointerGesture, UI_ACTIVE_POINTER_GESTURE_LIMIT,
};
pub use stop::{UiPointerGestureStop, UiPointerGestureStopReason};

pub(crate) use pointer::{
    UiScrollChromeLatch, UiScrollChromeLatchDenial, UiScrollChromeLatchState,
    UiScrollChromePendingCapture,
};

#[allow(
    unused_imports,
    reason = "milestone 3.16 Gate 0 exposes the sealed pressed appearance contract internally"
)]
pub(crate) use pointer::{
    UiPointerGestureOutcome, UiPointerGestureRuntimeState, UiPointerGestureStateSnapshot,
    UiPreparedPointerGestureCancellation, UiPressedAppearanceClass,
    UiPressedAppearanceOwnerSnapshot, UiPressedAppearancePosture,
};
