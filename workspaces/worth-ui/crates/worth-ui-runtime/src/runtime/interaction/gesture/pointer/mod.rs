mod accessors;
mod cancellation;
pub(crate) use cancellation::UiPreparedPointerGestureCancellation;
mod appearance;
mod model;
mod presentation;
mod scroll_chrome_latch;
mod transition;

pub(crate) use scroll_chrome_latch::{
    UiScrollChromeLatch, UiScrollChromeLatchDenial, UiScrollChromeLatchState,
    UiScrollChromePendingCapture,
};

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
    pub(crate) fn has_appearance_records(&self) -> bool {
        self.appearance_enabled && !self.active.is_empty()
    }

    pub(crate) fn new(appearance_enabled: bool) -> Self {
        Self {
            active: BTreeMap::new(),
            counters: Default::default(),
            appearance_revision: 0,
            appearance_enabled,
            scroll_chrome: Default::default(),
        }
    }

    /// The thumb drag in progress, if one is.
    pub(crate) fn scroll_chrome_latch(&self) -> Option<UiScrollChromeLatch> {
        self.scroll_chrome.held()
    }

    /// The latch slot as read-only state, for the lanes that only need to ask
    /// what a drag in progress already owns.
    pub(crate) const fn scroll_chrome_latch_state(&self) -> &UiScrollChromeLatchState {
        &self.scroll_chrome
    }

    pub(crate) fn scroll_chrome_capture_identity(
        &self,
    ) -> Option<(
        UiHostPointerIdentity,
        worth_ui_host_contract::UiHostPointerCaptureEpoch,
    )> {
        self.scroll_chrome
            .pending()
            .filter(|pending| !pending.released())
            .map(|pending| (pending.pointer(), pending.capture_epoch()))
            .or_else(|| {
                self.scroll_chrome
                    .held()
                    .map(|held| (held.pointer(), held.capture_epoch()))
            })
    }

    /// The latch slot, for the press, move and release lane that owns a drag.
    pub(crate) fn scroll_chrome_latch_mut(
        &mut self,
    ) -> &mut scroll_chrome_latch::UiScrollChromeLatchState {
        &mut self.scroll_chrome
    }

    pub(crate) fn process_report(
        &mut self,
        core: worth_ui_host_contract::UiHostObservationCanonicalCore,
        report: &worth_ui_host_contract::UiHostObservationReport,
        kind: Option<crate::runtime::interaction::UiPrimaryPointerKind>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Vec<UiPointerGestureOutcome> {
        self.process_pointer_report(core, report, kind, mounted, work)
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
        // A drag is presented under one binding; when that binding is gone the
        // rectangles the drag was mapping through are gone with it.
        self.scroll_chrome.cancel_binding(binding);
        self.cancel_where(|active| active.target.binding() == binding, reason)
    }

    pub(crate) fn cancel_instance(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        // A thumb drag ends with the occurrence it was dragging: the region is
        // gone, so the offset the drag was placing has no owner left.
        self.scroll_chrome.cancel_instance(instance);
        self.cancel_where(
            |active| active.target.mounted_instance() == instance,
            reason,
        )
    }

    pub(crate) fn cancel_all(
        &mut self,
        reason: UiPointerGestureStopReason,
    ) -> Vec<super::UiPointerGestureStop> {
        // Modality changes and lost focus reach every gesture, and a captured
        // thumb is a gesture: capture introduces no lifetime of its own.
        self.scroll_chrome.cancel();
        let prepared = self.prepare_cancel_all(reason);
        self.commit_prepared_cancellation(prepared)
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
