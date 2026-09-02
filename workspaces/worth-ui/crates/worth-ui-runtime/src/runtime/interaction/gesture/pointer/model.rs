use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiHostObservationSequence, UiHostObservationTimeBasis, UiHostPointerButton,
    UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
};

use super::super::UiPointerGestureStop;
use crate::runtime::interaction::targeting::{
    UiPointerGestureContinuityKind, UiPresentedInteractionTarget,
};

pub const UI_ACTIVE_POINTER_GESTURE_LIMIT: usize = 16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPointerGestureLifecycleCounters {
    pub(crate) button_reports: u64,
    pub(crate) gestures_started: u64,
    pub(crate) gestures_completed: u64,
    pub(crate) stop_outcomes: u64,
    pub(crate) active_gestures_settled: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerGestureStateSnapshot {
    pub(crate) active_gestures: usize,
    pub(crate) counters: UiPointerGestureLifecycleCounters,
}

#[derive(Debug)]
pub struct UiPointerGesturePressReceipt {
    pub(super) pointer: UiHostPointerIdentity,
    pub(super) capture_epoch: UiHostPointerCaptureEpoch,
    pub(super) button: UiHostPointerButton,
    pub(super) sequence: UiHostObservationSequence,
    pub(super) time_basis: UiHostObservationTimeBasis,
    pub(super) position: worth_ui_host_contract::UiHostSurfacePosition,
    pub(super) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(super) pointer_device_kind: UiHostPointerDeviceKind,
}

#[derive(Debug)]
pub struct UiTargetedPointerGesture {
    pub(super) pointer: UiHostPointerIdentity,
    pub(super) capture_epoch: UiHostPointerCaptureEpoch,
    pub(super) button: UiHostPointerButton,
    pub(super) press_sequence: UiHostObservationSequence,
    pub(super) press_time_basis: UiHostObservationTimeBasis,
    pub(super) release_sequence: UiHostObservationSequence,
    pub(super) release_time_basis: UiHostObservationTimeBasis,
    pub(super) pressed: UiPresentedInteractionTarget,
    pub(super) released: UiPresentedInteractionTarget,
    pub(super) continuity: UiPointerGestureContinuityKind,
    pub(super) continuity_witness_digest: u64,
    pub(super) pointer_device_kind: UiHostPointerDeviceKind,
}

#[derive(Debug)]
pub(crate) enum UiPointerGestureOutcome {
    Pressed(UiPointerGesturePressReceipt),
    Completed(UiTargetedPointerGesture),
    Stopped(UiPointerGestureStop),
}

pub(crate) struct UiPointerGestureRuntimeState {
    pub(super) active: BTreeMap<UiHostPointerIdentity, UiActivePointerGesture>,
    pub(super) counters: UiPointerGestureLifecycleCounters,
    pub(super) appearance_revision: u64,
    pub(super) appearance_enabled: bool,
}

pub(super) struct UiActivePointerGesture {
    pub(super) kind: crate::runtime::interaction::UiPrimaryPointerKind,
    pub(super) capture_epoch: UiHostPointerCaptureEpoch,
    pub(super) button: UiHostPointerButton,
    pub(super) press_sequence: UiHostObservationSequence,
    pub(super) press_time_basis: UiHostObservationTimeBasis,
    pub(super) target: UiPresentedInteractionTarget,
    pub(super) position: worth_ui_host_contract::UiHostSurfacePosition,
    pub(super) inside: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 classifies pressed posture before role resolution consumes it"
)]
pub(crate) enum UiPressedAppearanceClass {
    ArmedInside,
    CapturedOutside,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals pressed posture before role resolution consumes it"
)]
pub(crate) struct UiPressedAppearancePosture {
    pointer: UiHostPointerIdentity,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    target: worth_ui_host_contract::UiMountedInstanceIdentity,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    class: UiPressedAppearanceClass,
    owner_revision: u64,
    press_sequence: UiHostObservationSequence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals pressed snapshots before role resolution consumes them"
)]
pub(crate) struct UiPressedAppearanceOwnerSnapshot {
    owner_revision: u64,
    postures: Box<[UiPressedAppearancePosture]>,
}

#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals pressed snapshots before role resolution consumes them"
)]
impl UiPressedAppearanceOwnerSnapshot {
    pub(super) fn seal(state: &UiPointerGestureRuntimeState) -> Self {
        let postures = state
            .active
            .iter()
            .map(|(pointer, active)| UiPressedAppearancePosture {
                pointer: *pointer,
                presentation: active.target.presentation(),
                target: active.target.mounted_instance(),
                node_receipt: active.target.node_receipt(),
                class: if active.inside {
                    UiPressedAppearanceClass::ArmedInside
                } else {
                    UiPressedAppearanceClass::CapturedOutside
                },
                owner_revision: state.appearance_revision,
                press_sequence: active.press_sequence,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            owner_revision: state.appearance_revision,
            postures,
        }
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn postures(&self) -> &[UiPressedAppearancePosture] {
        &self.postures
    }
}

#[allow(
    dead_code,
    reason = "Gate 0 freezes read-only pressed-axis products before Gate 1 adapters"
)]
impl UiPressedAppearancePosture {
    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub(crate) const fn target(self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.target
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub(crate) const fn class(self) -> UiPressedAppearanceClass {
        self.class
    }
    pub(crate) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }
    pub(crate) const fn press_sequence(self) -> UiHostObservationSequence {
        self.press_sequence
    }
}
