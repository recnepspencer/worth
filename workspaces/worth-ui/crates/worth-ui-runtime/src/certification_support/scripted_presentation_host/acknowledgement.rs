//! What a script tells the host to acknowledge. The host mints the completion
//! itself, from the exact work the runtime issued, so a script cannot claim a
//! surface, frame, or attempt it was never asked to present.

use worth_ui_host_contract::{
    UiHostPresentationCompletionToken, UiHostPresentationCostReport, UiHostPresentationEpoch,
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode, UiMountedCompletedEffects,
    UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView,
    UiMountedSurfacePresentationCompletion,
};

pub struct ScriptedPresentationAcknowledgement {
    mode: UiHostSurfacePresentationMode,
    epoch: UiHostPresentationEpoch,
    effects: UiMountedCompletedEffects,
    cost: UiHostPresentationCostReport,
}

pub enum ScriptedPresentationOutcome {
    RejectedBeforeEffects(UiHostSurfacePresentationDenial),
    Presented(ScriptedPresentationAcknowledgement),
    /// The host acknowledges a view it built itself from the issued view's own
    /// identity: the same attempt, requirement and frame, under a seal no
    /// runtime lease issued. Only the runtime's own issuance is admissible.
    PresentedFromForeignView(ScriptedPresentationAcknowledgement),
    PresentationIndeterminate,
}

impl ScriptedPresentationAcknowledgement {
    pub fn new(
        mode: UiHostSurfacePresentationMode,
        epoch: UiHostPresentationEpoch,
        effects: UiMountedCompletedEffects,
        cost: UiHostPresentationCostReport,
    ) -> Self {
        Self {
            mode,
            epoch,
            effects,
            cost,
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the scripted host acknowledges the presentation it scripted"
    )]
    pub(super) fn acknowledge_view(
        self,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> UiMountedSurfacePresentationCompletion {
        view.acknowledge_presented(self.mode, self.epoch, self.effects, self.cost)
    }

    pub(super) fn acknowledge_foreign_view(
        self,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> UiMountedSurfacePresentationCompletion {
        let foreign =
            UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
                authority: std::rc::Rc::new(()),
                host_session_identity: view.host_session_identity(),
                protocol: view.protocol(),
                capability_generation: view.capability_generation(),
                capability_profile_digest: view.capability_profile_digest(),
                attempt: view.attempt(),
                deadline: view.deadline(),
                requirement: view.requirement(),
                presentation_work: view.presentation_work(),
                appearance_work: view.appearance_work(),
                qualified_text: &(),
                text_raster_work: view.text_raster_work(),
            });
        self.acknowledge_view(&foreign)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the scripted host acknowledges the presentation it scripted"
    )]
    pub(super) fn acknowledge_token(
        self,
        token: UiHostPresentationCompletionToken,
    ) -> UiMountedSurfacePresentationCompletion {
        token.acknowledge_presented(self.mode, self.epoch, self.effects, self.cost)
    }
}
