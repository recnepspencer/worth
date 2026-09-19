use super::UiPreparedMountedFrame;

impl UiPreparedMountedFrame {
    pub(crate) fn begin_appearance_lifecycle(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<(), crate::mounting::projection::UiMountedProjectionDenial> {
        self.frame
            .begin_appearance_lifecycle(session, generation, graph)
    }

    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.lower_appearance(presentation, profile)
    }

    pub(crate) fn stage_untrusted_pointer_affordance_for_test(
        &mut self,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        self.frame
            .stage_untrusted_pointer_affordance_for_test(snapshot, mounted)
    }

    pub(crate) fn clear_pointer_affordance(&mut self) {
        self.frame.clear_pointer_affordance()
    }
}
