use super::UiRetainedPresentedFrame;
use std::rc::Rc;

/// Only successful mounting/host settlement hands this evidence to interaction.
/// Rc snapshots preserve the exact prior physical sample across same-frame ticks.
#[derive(Clone)]
pub(crate) struct UiCommittedPresentedHitTransition {
    previous: Option<Rc<UiRetainedPresentedFrame>>,
    current: Rc<UiRetainedPresentedFrame>,
}

impl std::fmt::Debug for UiCommittedPresentedHitTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiCommittedPresentedHitTransition")
            .field("frame", &self.current.frame())
            .finish_non_exhaustive()
    }
}

impl UiCommittedPresentedHitTransition {
    pub(super) fn new(
        previous: Option<Rc<UiRetainedPresentedFrame>>,
        current: Rc<UiRetainedPresentedFrame>,
    ) -> Self {
        Self { previous, current }
    }

    pub(crate) fn changes(&self) -> crate::mounting::presented_hit_index::UiPresentedHitChanges {
        crate::mounting::presented_hit_index::UiPresentedHitChanges::between(
            self.previous
                .as_ref()
                .map(|frame| frame.hit_index())
                .unwrap_or_default(),
            self.current.hit_index(),
        )
    }

    pub(crate) fn presentations(
        &self,
    ) -> Vec<(
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        worth_ui_host_contract::UiHostObservationPresentationBasis,
    )> {
        self.current.hit_presentations()
    }
}
