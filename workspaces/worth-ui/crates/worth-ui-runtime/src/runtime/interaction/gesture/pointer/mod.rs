mod accessors;
mod model;
mod presentation;
mod transition;

use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiHostObservationSequence, UiHostPointerIdentity, UiSurfaceBindingGeneration,
};

use super::UiPointerGestureStopReason;
use model::UiActivePointerGesture;
#[allow(
    unused_imports,
    reason = "milestone 3.16 Gate 0 exposes the sealed pressed appearance contract internally"
)]
pub(crate) use model::{
    UiPointerGestureOutcome, UiPointerGestureRuntimeState, UiPointerGestureStateSnapshot,
    UiPressedAppearanceClass, UiPressedAppearanceOwnerSnapshot, UiPressedAppearancePosture,
};
pub use model::{
    UiPointerGesturePressReceipt, UiTargetedPointerGesture, UI_ACTIVE_POINTER_GESTURE_LIMIT,
};

impl UiPointerGestureRuntimeState {
    pub(crate) fn new(appearance_enabled: bool) -> Self {
        Self {
            active: BTreeMap::new(),
            counters: Default::default(),
            appearance_revision: 0,
            appearance_enabled,
        }
    }

    pub(crate) fn process_report(
        &mut self,
        core: worth_ui_host_contract::UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        kind: Option<crate::runtime::interaction::UiPrimaryPointerKind>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Vec<UiPointerGestureOutcome> {
        self.process_pointer_report(core, report, kind, mounted)
    }

    pub(crate) fn stop_pointer_for_denial(
        &mut self,
        pointer: UiHostPointerIdentity,
        sequence: UiHostObservationSequence,
        reason: UiPointerGestureStopReason,
    ) -> Vec<UiPointerGestureOutcome> {
        self.stop_active_pointer_for_denial(pointer, sequence, reason)
    }

    pub(crate) fn snapshot(&self) -> UiPointerGestureStateSnapshot {
        UiPointerGestureStateSnapshot {
            active_gestures: self.active.len(),
            counters: self.counters,
        }
    }

    #[allow(
        dead_code,
        reason = "milestone 3.16 Gate 0 exposes the owner snapshot only to the sealed close-turn lane"
    )]
    pub(crate) fn appearance_snapshot(&self) -> UiPressedAppearanceOwnerSnapshot {
        UiPressedAppearanceOwnerSnapshot::seal(self)
    }

    pub(crate) fn cancel_binding(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        self.cancel_where(|active| active.target.binding() == binding, reason)
    }

    pub(crate) fn cancel_instance(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        self.cancel_where(
            |active| active.target.mounted_instance() == instance,
            reason,
        )
    }

    pub(crate) fn cancel_all(
        &mut self,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        self.cancel_where(|_| true, reason)
    }

    fn cancel_where(
        &mut self,
        predicate: impl Fn(&UiActivePointerGesture) -> bool,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        let selected = take_matching(&mut self.active, predicate);
        if !selected.is_empty() {
            self.bump_appearance_revision();
        }
        self.counters.stop_outcomes = add(self.counters.stop_outcomes, selected.len());
        self.counters.active_gestures_settled =
            add(self.counters.active_gestures_settled, selected.len());
        selected
            .into_iter()
            .map(|(pointer, active)| {
                super::UiPointerGestureStop::new(
                    pointer,
                    active.capture_epoch,
                    active.button,
                    None,
                    true,
                    reason,
                )
            })
            .collect()
    }

    pub(super) fn bump_button_reports(&mut self) {
        self.counters.button_reports = next(self.counters.button_reports);
    }

    pub(super) fn bump_appearance_revision(&mut self) {
        if self.appearance_enabled {
            self.appearance_revision = next(self.appearance_revision);
        }
    }

    pub(crate) fn reconcile_appearance_enabled(&mut self, enabled: bool) {
        self.appearance_enabled = enabled;
        if !enabled {
            self.appearance_revision = 0;
        }
    }
}

fn take_matching(
    active: &mut BTreeMap<UiHostPointerIdentity, UiActivePointerGesture>,
    predicate: impl Fn(&UiActivePointerGesture) -> bool,
) -> Vec<(UiHostPointerIdentity, UiActivePointerGesture)> {
    let selected = active
        .iter()
        .filter_map(|(pointer, gesture)| predicate(gesture).then_some(*pointer))
        .collect::<Vec<_>>();
    selected
        .into_iter()
        .map(|pointer| {
            let gesture = active
                .remove(&pointer)
                .expect("the selected gesture remains active");
            (pointer, gesture)
        })
        .collect()
}

fn next(value: u64) -> u64 {
    value
        .checked_add(1)
        .expect("host observation sequence exhaustion stops before counter overflow")
}

fn add(value: u64, count: usize) -> u64 {
    value
        .checked_add(u64::try_from(count).expect("bounded gesture count fits u64"))
        .expect("host observation sequence exhaustion stops before counter overflow")
}
