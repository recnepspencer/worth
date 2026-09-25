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
        let shielding_changed = self
            .current
            .hit_presentations()
            .into_iter()
            .map(|(_, presentation)| presentation.binding())
            .filter(|binding| {
                self.previous
                    .as_ref()
                    .and_then(|frame| frame.modal_input_admission(*binding))
                    != self.current.modal_input_admission(*binding)
            })
            .collect();
        crate::mounting::presented_hit_index::UiPresentedHitChanges::between(
            self.previous
                .as_ref()
                .map(|frame| frame.hit_index())
                .unwrap_or_default(),
            self.current.hit_index(),
        )
        .with_input_shielding_changed(shielding_changed)
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
