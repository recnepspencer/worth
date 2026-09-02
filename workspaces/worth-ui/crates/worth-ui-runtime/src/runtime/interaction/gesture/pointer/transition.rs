use worth_ui_host_contract::{
    UiHostObservationCanonicalCore, UiHostObservationPayload, UiHostObservationSequence,
    UiHostObservationTimeBasis, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerIdentity,
};

use super::model::{
    UiActivePointerGesture, UiPointerGestureOutcome, UiPointerGesturePressReceipt,
    UiPointerGestureRuntimeState, UiTargetedPointerGesture, UI_ACTIVE_POINTER_GESTURE_LIMIT,
};
use super::next;
use crate::runtime::interaction::gesture::{UiPointerGestureStop, UiPointerGestureStopReason};
use crate::runtime::interaction::targeting::{
    issue_continuity, resolve_presented_target, UiPointerGestureContinuityDenial,
};

#[derive(Clone, Copy)]
struct UiPointerButtonReport<'world> {
    core: UiHostObservationCanonicalCore,
    sequence: UiHostObservationSequence,
    time_basis: UiHostObservationTimeBasis,
    pointer: UiHostPointerIdentity,
    capture_epoch: UiHostPointerCaptureEpoch,
    button: UiHostPointerButton,
    position: worth_ui_host_contract::UiHostSurfacePosition,
    kind: crate::runtime::interaction::UiPrimaryPointerKind,
    mounted: &'world crate::mounting::WorthUiMountedSessionState,
}

impl UiPointerGestureRuntimeState {
    pub(super) fn process_pointer_report(
        &mut self,
        core: UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        kind: crate::runtime::interaction::UiPrimaryPointerKind,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Vec<UiPointerGestureOutcome> {
        match report.payload() {
            UiHostObservationPayload::PointerButton {
                pointer,
                capture_epoch,
                button,
                transition,
                position,
            } => {
                self.bump_button_reports();
                let input = UiPointerButtonReport {
                    core,
                    sequence: report.sequence(),
                    time_basis: report.time_basis(),
                    pointer: *pointer,
                    capture_epoch: *capture_epoch,
                    button: *button,
                    position: *position,
                    kind,
                    mounted,
                };
                vec![match transition {
                    UiHostPointerButtonTransition::Pressed => self.press(input),
                    UiHostPointerButtonTransition::Released => self.release(input),
                }]
            }
            UiHostObservationPayload::PointerMotion {
                pointer,
                capture_epoch,
                position,
                ..
            } => self.motion(
                core,
                report.sequence(),
                *pointer,
                *capture_epoch,
                *position,
                kind,
                mounted,
            ),
            UiHostObservationPayload::WindowFocus { focused: false, .. } => {
                self.focus_loss(report.sequence())
            }
            _ => Vec::new(),
        }
    }

    fn press(&mut self, input: UiPointerButtonReport<'_>) -> UiPointerGestureOutcome {
        if input.button != UiHostPointerButton::Primary {
            return self.failed_stop(
                input,
                UiPointerGestureStopReason::UnsupportedButton(input.button),
            );
        }
        if let Some(active) = self.active.remove(&input.pointer) {
            self.bump_appearance_revision();
            let reason = capture_change_reason(&active, input.capture_epoch)
                .or_else(|| pointer_kind_change_reason(&active, input.kind))
                .unwrap_or(UiPointerGestureStopReason::DuplicatePress);
            return self.active_stop(input.pointer, active, input.sequence, reason);
        }
        if self.active.len() >= UI_ACTIVE_POINTER_GESTURE_LIMIT {
            return self.failed_stop(
                input,
                UiPointerGestureStopReason::CapacityExceeded {
                    limit: UI_ACTIVE_POINTER_GESTURE_LIMIT,
                },
            );
        }
        let target = match resolve_presented_target(
            input.mounted,
            input.core.presentation(),
            input.position,
        ) {
            Ok(target) => target,
            Err(denial) => {
                return self.failed_stop(input, UiPointerGestureStopReason::Targeting(denial))
            }
        };
        let target_view = target.view();
        let active = UiActivePointerGesture {
            kind: input.kind,
            capture_epoch: input.capture_epoch,
            button: input.button,
            press_sequence: input.sequence,
            press_time_basis: input.time_basis,
            target,
            position: input.position,
            inside: true,
        };
        self.active.insert(input.pointer, active);
        self.bump_appearance_revision();
        self.counters.gestures_started = next(self.counters.gestures_started);
        UiPointerGestureOutcome::Pressed(UiPointerGesturePressReceipt {
            pointer: input.pointer,
            capture_epoch: input.capture_epoch,
            button: input.button,
            sequence: input.sequence,
            time_basis: input.time_basis,
            position: input.position,
            target: target_view,
            pointer_device_kind: input.kind.host_kind(),
        })
    }

    fn release(&mut self, input: UiPointerButtonReport<'_>) -> UiPointerGestureOutcome {
        let Some(active) = self.active.remove(&input.pointer) else {
            return self.failed_stop(input, UiPointerGestureStopReason::NoActiveGesture);
        };
        self.bump_appearance_revision();
        if let Some(reason) = capture_change_reason(&active, input.capture_epoch) {
            return self.active_stop(input.pointer, active, input.sequence, reason);
        }
        if let Some(reason) = pointer_kind_change_reason(&active, input.kind) {
            return self.active_stop(input.pointer, active, input.sequence, reason);
        }
        if active.button != input.button {
            let reason = UiPointerGestureStopReason::ButtonChanged {
                expected: active.button,
                observed: input.button,
            };
            return self.active_stop(input.pointer, active, input.sequence, reason);
        }
        let released = match resolve_presented_target(
            input.mounted,
            input.core.presentation(),
            input.position,
        ) {
            Ok(target) => target,
            Err(denial) => {
                return self.active_stop(
                    input.pointer,
                    active,
                    input.sequence,
                    UiPointerGestureStopReason::Targeting(denial),
                )
            }
        };
        let witness = match issue_continuity(&active.target, &released) {
            Ok(witness) => witness,
            Err(denial) => {
                return self.active_stop(
                    input.pointer,
                    active,
                    input.sequence,
                    map_continuity_denial(denial),
                )
            }
        };
        self.counters.gestures_completed = next(self.counters.gestures_completed);
        self.counters.active_gestures_settled = next(self.counters.active_gestures_settled);
        UiPointerGestureOutcome::Completed(UiTargetedPointerGesture {
            pointer: input.pointer,
            capture_epoch: input.capture_epoch,
            button: input.button,
            press_sequence: active.press_sequence,
            press_time_basis: active.press_time_basis,
            release_sequence: input.sequence,
            release_time_basis: input.time_basis,
            pressed: active.target,
            released,
            continuity: witness.kind(),
            continuity_witness_digest: witness.digest(),
            pointer_device_kind: active.kind.host_kind(),
        })
    }

    fn motion(
        &mut self,
        core: UiHostObservationCanonicalCore,
        sequence: UiHostObservationSequence,
        pointer: UiHostPointerIdentity,
        observed: UiHostPointerCaptureEpoch,
        position: worth_ui_host_contract::UiHostSurfacePosition,
        kind: crate::runtime::interaction::UiPrimaryPointerKind,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Vec<UiPointerGestureOutcome> {
        let Some(active_capture_epoch) = self.active.get(&pointer).map(|active| {
            (
                active.capture_epoch,
                (
                    active.target.surface(),
                    active.target.binding(),
                    active.target.mounted_instance(),
                    active.target.node_receipt(),
                ),
                active.inside,
                active.kind,
            )
        }) else {
            return Vec::new();
        };
        if active_capture_epoch.3 != kind {
            let active = self
                .active
                .remove(&pointer)
                .expect("the active pointer was just observed");
            self.bump_appearance_revision();
            let reason = pointer_kind_change_reason(&active, kind)
                .expect("the observed pointer kind changed");
            return vec![self.active_stop(pointer, active, sequence, reason)];
        }
        if active_capture_epoch.0 == observed {
            let active_target = active_capture_epoch.1;
            self.active
                .get_mut(&pointer)
                .expect("active gesture remains present")
                .position = position;
            if !self.appearance_enabled {
                return Vec::new();
            }
            let inside = resolve_presented_target(mounted, core.presentation(), position)
                .is_ok_and(|target| {
                    (
                        target.surface(),
                        target.binding(),
                        target.mounted_instance(),
                        target.node_receipt(),
                    ) == active_target
                });
            if inside != active_capture_epoch.2 {
                self.active
                    .get_mut(&pointer)
                    .expect("active gesture remains present")
                    .inside = inside;
                self.bump_appearance_revision();
            }
            return Vec::new();
        }
        let active = self
            .active
            .remove(&pointer)
            .expect("the active pointer was just observed");
        self.bump_appearance_revision();
        let reason = UiPointerGestureStopReason::CaptureChanged {
            expected: active.capture_epoch,
            observed,
        };
        vec![self.active_stop(pointer, active, sequence, reason)]
    }

    fn focus_loss(&mut self, sequence: UiHostObservationSequence) -> Vec<UiPointerGestureOutcome> {
        let active = std::mem::take(&mut self.active);
        if !active.is_empty() {
            self.bump_appearance_revision();
        }
        active
            .into_iter()
            .map(|(pointer, active)| {
                self.active_stop(
                    pointer,
                    active,
                    sequence,
                    UiPointerGestureStopReason::FocusLost,
                )
            })
            .collect()
    }

    fn failed_stop(
        &mut self,
        input: UiPointerButtonReport<'_>,
        reason: UiPointerGestureStopReason,
    ) -> UiPointerGestureOutcome {
        self.counters.stop_outcomes = next(self.counters.stop_outcomes);
        UiPointerGestureOutcome::Stopped(UiPointerGestureStop::new(
            input.pointer,
            input.capture_epoch,
            input.button,
            Some(input.sequence),
            false,
            reason,
        ))
    }

    fn active_stop(
        &mut self,
        pointer: UiHostPointerIdentity,
        active: UiActivePointerGesture,
        sequence: UiHostObservationSequence,
        reason: UiPointerGestureStopReason,
    ) -> UiPointerGestureOutcome {
        self.counters.stop_outcomes = next(self.counters.stop_outcomes);
        self.counters.active_gestures_settled = next(self.counters.active_gestures_settled);
        UiPointerGestureOutcome::Stopped(UiPointerGestureStop::new(
            pointer,
            active.capture_epoch,
            active.button,
            Some(sequence),
            true,
            reason,
        ))
    }
}

fn capture_change_reason(
    active: &UiActivePointerGesture,
    observed: UiHostPointerCaptureEpoch,
) -> Option<UiPointerGestureStopReason> {
    (active.capture_epoch != observed).then_some(UiPointerGestureStopReason::CaptureChanged {
        expected: active.capture_epoch,
        observed,
    })
}

fn pointer_kind_change_reason(
    active: &UiActivePointerGesture,
    observed: crate::runtime::interaction::UiPrimaryPointerKind,
) -> Option<UiPointerGestureStopReason> {
    (active.kind != observed).then_some(UiPointerGestureStopReason::PointerDeviceKindChanged {
        expected: active.kind.host_kind(),
        observed: observed.host_kind(),
    })
}

fn map_continuity_denial(denial: UiPointerGestureContinuityDenial) -> UiPointerGestureStopReason {
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
