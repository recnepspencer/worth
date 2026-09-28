use super::UiMountedPresentationCoordinator;
use crate::mounting::presentation::work_producer::UiShownOwnPaint;

impl UiMountedPresentationCoordinator {
    /// How the host shows what `instance` paints as itself on the displayed
    /// `presentation`, each command carrying `beside`, a rect drawn in the
    /// same layout. `None` when the presentation state holds another frame.
    pub(in crate::mounting) fn shown_own_paint(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        beside: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Option<Vec<UiShownOwnPaint>> {
        let state = self.presentation_states.get(&presentation.binding())?;
        (state.frame() == presentation.frame()).then(|| state.shown_own_paint(instance, beside))
    }

    /// Whether the frame the displayed `presentation` shows placed the
    /// Scroll group `target`. `false` when the presentation state holds
    /// another frame.
    pub(in crate::mounting) fn placed_scroll_group(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> bool {
        self.presentation_states
            .get(&presentation.binding())
            .is_some_and(|state| {
                state.frame() == presentation.frame() && state.placed_scroll_group(target)
            })
    }
}
