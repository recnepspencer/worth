use super::{model::UiPointerGestureStateSnapshot, UiPointerGestureRuntimeState};
use crate::runtime::interaction::{UiPointerGestureStop, UiPointerGestureStopReason};

pub(crate) struct UiPreparedPointerGestureCancellation {
    predecessor: UiPointerGestureStateSnapshot,
    predecessor_revision: u64,
    predecessor_enabled: bool,
    successor: UiPointerGestureRuntimeState,
    stops: Vec<UiPointerGestureStop>,
}

impl UiPreparedPointerGestureCancellation {
    pub(crate) fn appearance_snapshot(&self) -> super::UiPressedAppearanceOwnerSnapshot {
        self.successor.appearance_snapshot()
    }
}

impl UiPointerGestureRuntimeState {
    pub(crate) fn prepare_cancel_all(
        &self,
        reason: UiPointerGestureStopReason,
    ) -> UiPreparedPointerGestureCancellation {
        self.prepare_cancel_all_with_appearance(reason, self.appearance_enabled)
    }

    pub(crate) fn prepare_cancel_all_with_appearance(
        &self,
        reason: UiPointerGestureStopReason,
        enabled: bool,
    ) -> UiPreparedPointerGestureCancellation {
        let stops = self
            .active
            .iter()
            .map(|(pointer, active)| {
                UiPointerGestureStop::new(
                    *pointer,
                    active.capture_epoch,
                    active.button,
                    None,
                    true,
                    reason,
                )
            })
            .collect::<Vec<_>>();
        let mut successor = Self::new(self.appearance_enabled);
        successor.counters = self.counters;
        successor.appearance_revision = self.appearance_revision;
        successor.reconcile_appearance_enabled(enabled);
        if !stops.is_empty() {
            successor.bump_appearance_revision();
        }
        successor.counters.stop_outcomes =
            super::add(successor.counters.stop_outcomes, stops.len());
        successor.counters.active_gestures_settled =
            super::add(successor.counters.active_gestures_settled, stops.len());
        UiPreparedPointerGestureCancellation {
            predecessor: self.snapshot(),
            predecessor_revision: self.appearance_revision,
            predecessor_enabled: self.appearance_enabled,
            successor,
            stops,
        }
    }

    pub(crate) fn admits_prepared_cancellation(
        &self,
        prepared: &UiPreparedPointerGestureCancellation,
    ) -> bool {
        self.snapshot() == prepared.predecessor
            && self.appearance_revision == prepared.predecessor_revision
            && self.appearance_enabled == prepared.predecessor_enabled
    }

    pub(crate) fn commit_prepared_cancellation(
        &mut self,
        prepared: UiPreparedPointerGestureCancellation,
    ) -> Vec<UiPointerGestureStop> {
        assert!(
            self.admits_prepared_cancellation(&prepared),
            "prepared pointer cancellation retains its owner revision"
        );
        *self = prepared.successor;
        prepared.stops
    }
}
