use super::*;

impl UiInteractionRuntimeState {
    pub(crate) fn observe_presented_hit_transition(
        &mut self,
        transition: &crate::mounting::UiCommittedPresentedHitTransition,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) {
        if self
            .pointer_presence
            .as_ref()
            .is_none_or(|owner| owner.pointer_count() == 0)
            && !self.pointer.has_appearance_records()
        {
            return;
        }
        let changes = transition.changes();
        let presentations = transition.presentations();
        // A denied predecessor cannot authorize receipt-only freshness. Retry is
        // owner-local and persists when an event has no matching pointer records.
        let hover_retry = self
            .presentation_refresh
            .is_some_and(|last| last.hover_retry);
        let pressed_retry = self
            .presentation_refresh
            .is_some_and(|last| last.pressed_retry);
        let mut hover_neighborhood_work = Default::default();
        let mut pressed_neighborhood_work = Default::default();
        let hover = self
            .pointer_presence
            .as_mut()
            .map_or(Ok(Default::default()), |owner| {
                owner.refresh_hit_transition(
                    &changes,
                    hover_retry,
                    &mut hover_neighborhood_work,
                    &presentations,
                    mounted,
                )
            });
        let pressed = self.pointer.refresh_hit_transition(
            &changes,
            pressed_retry,
            &mut pressed_neighborhood_work,
            &presentations,
            mounted,
        );
        self.presentation_refresh = Some(super::super::UiInteractionPresentationRefreshSnapshot {
            hover_retry: hover.is_err()
                || hover_retry && hover.is_ok_and(|report| report.unmatched != 0),
            pressed_retry: pressed.is_err()
                || pressed_retry && pressed.is_ok_and(|report| report.unmatched != 0),
            hover_neighborhood_work,
            pressed_neighborhood_work,
            hover,
            pressed,
            comparison_steps: changes.comparison_steps(),
            comparison_key_probes: changes.comparison_key_probes(),
        });
    }

    pub(crate) const fn presentation_refresh_snapshot(
        &self,
    ) -> Option<super::super::UiInteractionPresentationRefreshSnapshot> {
        self.presentation_refresh
    }
}
