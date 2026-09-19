use worth_ui_host_contract::{
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostPointerButton,
    UiHostPointerCaptureEpoch, UiHostPointerIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerGestureStopReason {
    UnsupportedButton(UiHostPointerButton),
    DuplicatePress,
    CapacityExceeded {
        limit: usize,
    },
    NoActiveGesture,
    CaptureChanged {
        expected: UiHostPointerCaptureEpoch,
        observed: UiHostPointerCaptureEpoch,
    },
    PointerDeviceKindChanged {
        expected: worth_ui_host_contract::UiHostPointerDeviceKind,
        observed: worth_ui_host_contract::UiHostPointerDeviceKind,
    },
    MissingPointerDeviceKind,
    ButtonChanged {
        expected: UiHostPointerButton,
        observed: UiHostPointerButton,
    },
    Targeting(super::super::targeting::UiInteractionTargetingDenial),
    PresentationDidNotAdvance,
    SurfaceChanged,
    BindingChanged,
    MountedIncarnationChanged,
    TargetChangedWithinPresentation,
    PointerButtonLoss {
        affected: UiHostObservationSequenceRange,
    },
    FocusLost,
    InvalidObservation,
    ObservationQuarantined,
    SurfaceRebound,
    MountedInstanceRemoved,
    ApplicationRebound,
    Shutdown,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiPointerGestureStop {
    pointer: UiHostPointerIdentity,
    capture_epoch: UiHostPointerCaptureEpoch,
    button: UiHostPointerButton,
    triggering_sequence: Option<UiHostObservationSequence>,
    settled_active_gesture: bool,
    reason: UiPointerGestureStopReason,
}

impl UiPointerGestureStop {
    pub(super) const fn new(
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
        button: UiHostPointerButton,
        triggering_sequence: Option<UiHostObservationSequence>,
        settled_active_gesture: bool,
        reason: UiPointerGestureStopReason,
    ) -> Self {
        Self {
            pointer,
            capture_epoch,
            button,
            triggering_sequence,
            settled_active_gesture,
            reason,
        }
    }

    pub const fn pointer(&self) -> UiHostPointerIdentity {
        self.pointer
    }

    pub const fn capture_epoch(&self) -> UiHostPointerCaptureEpoch {
        self.capture_epoch
    }

    pub const fn button(&self) -> UiHostPointerButton {
        self.button
    }

    pub const fn triggering_sequence(&self) -> Option<UiHostObservationSequence> {
        self.triggering_sequence
    }

    pub const fn settled_active_gesture(&self) -> bool {
        self.settled_active_gesture
    }

    pub const fn reason(&self) -> UiPointerGestureStopReason {
        self.reason
    }
}
