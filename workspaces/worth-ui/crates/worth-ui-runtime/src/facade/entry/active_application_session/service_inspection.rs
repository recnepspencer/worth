impl super::WorthUiActiveApplicationSession {
    pub fn appearance_inspection_world(
        &self,
        surface: crate::facade::mounted::UiSemanticSurfaceIdentity,
    ) -> worth_ui_inspection::UiAppearanceInspectionWorld {
        self.appearance_inspection
            .current_world(surface.diagnostic_value())
    }

    pub fn why_appearance(
        &self,
        query: worth_ui_inspection::UiAppearanceInspectionQuery,
    ) -> worth_ui_inspection::UiAppearanceInspectionOutcome {
        self.appearance_inspection.query(query)
    }

    pub fn why_portal_closed(
        &self,
    ) -> Option<worth_ui_inspection::UiPortalClosedInspectionSummary> {
        crate::inspection::service::why_portal_closed(self.portal.as_ref())
    }

    pub fn why_focus_moved(&self) -> Option<worth_ui_inspection::UiFocusMovedInspectionSummary> {
        crate::inspection::service::why_focus_moved(self.focus.as_ref())
    }

    pub fn why_focus_restoration_failed(
        &self,
    ) -> Option<worth_ui_inspection::UiFocusRestorationFailedInspectionSummary> {
        crate::inspection::service::why_focus_restoration_failed(self.focus.as_ref())
    }

    pub fn why_motion_interrupted(
        &self,
    ) -> Option<worth_ui_inspection::UiMotionInterruptedInspectionSummary> {
        crate::inspection::service::why_motion_interrupted(self.motion.as_ref())
    }

    pub fn why_scroll_owner(&self) -> Option<worth_ui_inspection::UiScrollOwnerInspectionSummary> {
        crate::inspection::service::why_scroll_owner(self.scroll.as_ref())
    }

    pub fn why_selection_dropped(
        &self,
    ) -> Option<worth_ui_inspection::UiSelectionDroppedInspectionSummary> {
        crate::inspection::service::why_selection_dropped(self.selection.as_ref())
    }

    pub fn why_command_won(&self) -> Option<worth_ui_inspection::UiCommandWonInspectionSummary> {
        crate::inspection::service::why_command_won(self.command_routing.as_ref())
    }

    pub fn runtime_service_resource_census(
        &self,
    ) -> worth_ui_inspection::UiRuntimeServiceResourceCensus {
        crate::inspection::service::resource_census(
            crate::inspection::service::UiRuntimeServiceResourceOwnerView {
                portal: self.portal.as_ref(),
                focus: self.focus.as_ref(),
                motion: self.motion.as_ref(),
                scroll: self.scroll.as_ref(),
                selection: self.selection.as_ref(),
                command: self.command_routing.as_ref(),
                proposal_counts: self.application.service_proposal_resource_counts(),
                portal_exit_counts: self.portal_exit_retention.resource_counts(),
            },
        )
    }
}
