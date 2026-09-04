use super::{WorthUiActiveApplicationSession, WorthUiRuntimeShutdownReceipt};

impl WorthUiActiveApplicationSession {
    pub fn shutdown(mut self) -> WorthUiRuntimeShutdownReceipt {
        self.shutdown_portal_exit_retention();
        self.clear_authored_overlay_bindings();
        let rebind = self.rebind.shutdown();
        let visual_capture = self.visual_captures.shutdown();
        let visual_overlay = self.visual_overlays.shutdown();
        let previous_input = self.interaction.active_input_binding();
        let interaction = self.interaction.shutdown();
        let focus_placement = self.mounted.shutdown_focus_placement();
        let _focus_resources_released = self
            .focus
            .as_mut()
            .map_or(0, crate::runtime::focus::UiFocusRuntimeState::shutdown);
        let portal = self.portal.as_mut().map_or_else(
            crate::runtime::portal::UiPortalShutdownReport::default,
            crate::runtime::portal::UiPortalRuntimeState::shutdown,
        );
        let _presentation_motion_tracks = self.mounted.shutdown_motion_sampling();
        let motion = self.motion.as_mut().map_or_else(
            crate::runtime::motion::UiMotionShutdownReport::default,
            crate::runtime::motion::UiMotionRuntimeState::shutdown,
        );
        debug_assert!(motion.final_census().is_zero());
        let scroll_owners_released = self
            .scroll
            .as_mut()
            .map_or(0, crate::runtime::scroll::UiScrollRuntimeState::shutdown);
        let selection_owners_released = self.selection.as_mut().map_or(
            0,
            crate::runtime::selection::UiSelectionRuntimeState::shutdown,
        );
        let command_routes_released = self.command_routing.as_mut().map_or(
            0,
            crate::runtime::command_routing::UiCommandRoutingRuntimeState::shutdown,
        );
        let terminal_service_owner_census = self.runtime_service_resource_census();
        self.clear_displaced_input_recipient(previous_input);
        let confirmation = self.intent_confirmation.shutdown();
        let (admission, execution) = self.intent_admission.shutdown(&mut self.intent_execution);
        let observation_resources = self.application.retire_observation_resources(
            crate::runtime::observation::UiObservationResourceRetirementCause::ApplicationShutdown,
        );
        let intent_evidence = self
            .intent_evidence
            .retire(worth_ui_inspection::UiIntentEvidenceRetirementCause::ApplicationShutdown);
        let final_intent_resource_census = self.intent_resource_census();
        debug_assert!(final_intent_resource_census.is_empty());
        let (mounted_presentation, outcomes, presentation_async_cleanup) =
            self.mounted.shutdown_presentation(&self.host_session);
        for outcome in outcomes {
            let _ = self.finish_mounted_presentation(outcome);
        }
        self.mounted.assert_shutdown_resolved();
        self.host_exchange.shutdown();
        let host_session_release = self.host_session.release_adapter_session();
        let host_session_recovery = matches!(
            host_session_release,
            worth_ui_host_contract::UiHostSessionReleaseOutcome::ReleaseIndeterminate(_)
        )
        .then(|| crate::facade::WorthUiHostSessionReleaseRecovery::retain(self.host_session));
        self.application
            .shutdown()
            .bind_visual_capture(visual_capture)
            .bind_visual_overlay(visual_overlay)
            .bind_mounted_presentation(mounted_presentation)
            .bind_presentation_async_cleanup(presentation_async_cleanup)
            .bind_host_session_release(host_session_release)
            .bind_host_session_recovery(host_session_recovery)
            .bind_interaction(interaction)
            .bind_focus_placement(focus_placement)
            .bind_portal(portal)
            .bind_motion(motion)
            .bind_scroll_owners_released(scroll_owners_released)
            .bind_selection_owners_released(selection_owners_released)
            .bind_command_routes_released(command_routes_released)
            .bind_intent_confirmation(confirmation)
            .bind_intent_admission(admission)
            .bind_intent_execution(execution)
            .bind_observation_resources(observation_resources)
            .bind_intent_evidence(intent_evidence)
            .bind_intent_resource_census(final_intent_resource_census)
            .bind_terminal_runtime_service_census(terminal_service_owner_census)
            .bind_rebind(rebind)
    }
}
