use super::{
    WorthUiActiveApplicationGenerationIdentity, WorthUiActiveApplicationSession,
    WorthUiActiveApplicationSessionIdentity, WorthUiApp,
};
use crate::runtime::WorthUiRuntime;

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn new(
        mut app: WorthUiApp,
        runtime: WorthUiRuntime,
        host_session: crate::facade::WorthUiHostSessionAuthority,
    ) -> Result<Self, crate::runtime::WorthUiRuntimeLaunchDenial> {
        let identity = WorthUiActiveApplicationSessionIdentity::from_host_session_value(
            host_session.identity().as_u64(),
        );
        let initial_generation = WorthUiActiveApplicationGenerationIdentity::current(
            identity,
            app.generation_identity(),
        );
        let mounted_frame_retention_budget = app.mounted_frame_retention_budget();
        let host_observation_capacity = app.host_observation_capacity();
        let visual_policy = app.visual_inspection_policy();
        let rebind_profile = app.prepared_authority().change_profile().rebind();
        let presentation_async = app.take_presentation_async_owner();
        let service_policy_plan = app.service_policy_plan();
        let mut dormant_portal_stack_ordinal_issuer =
            Some(crate::runtime::portal::UiPortalStackOrdinalIssuer::new());
        let appearance_theme_admission =
            if app
                .prepared_authority()
                .consumed_fact_index()
                .has_appearance_consumers()
            {
                let themes = app.capabilities().appearance_themes().ok_or(
                    crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceThemeAdmission(
                        crate::runtime::appearance::UiThemeCapabilityReceiptDenial::MissingBundle,
                    ),
                )?;
                let host_profile = host_session
                .capability_report()
                .appearance_profile()
                .ok_or(crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceThemeAdmission(
                    crate::runtime::appearance::UiThemeCapabilityReceiptDenial::MissingHostProfile,
                ))?;
                let admission =
                crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
                    themes,
                    themes.initial_definition_identity(),
                    app.capabilities().appearance_roles(),
                    host_profile,
                )
                .map_err(crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceThemeAdmission)?;
                Some(
                    admission
                        .prepare(
                            app.prepared_authority()
                                .consumed_fact_index()
                                .appearance_required_role_identities(),
                            WorthUiActiveApplicationGenerationIdentity::current(
                                identity,
                                app.generation_identity(),
                            ),
                        )
                        .map_err(
                            crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceThemeAdmission,
                        )?,
                )
            } else {
                None
            };
        let presentation =
            crate::runtime::presentation_state::UiApplicationPresentationState::activate(
                app.capabilities(),
            );
        let appearance_axis_demand = app
            .prepared_authority()
            .consumed_fact_index()
            .appearance_axis_demand();
        let intent_application_facts =
            crate::runtime::intent::UiIntentApplicationFactState::activate(
                app.prepared_authority().intent_application_fact_plan(),
                appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Validation),
            );
        if appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Focus)
            && service_policy_plan.focus().is_none()
        {
            return Err(
                crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceOwnerUnavailable(
                    worth_ui_dsl::UiAppearanceStateAxis::Focus,
                ),
            );
        }
        if appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Selection)
            && service_policy_plan.selection().is_none()
        {
            return Err(
                crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceOwnerUnavailable(
                    worth_ui_dsl::UiAppearanceStateAxis::Selection,
                ),
            );
        }
        let command_routing = crate::runtime::UiRuntimeServiceInstallation::from_optional(
            service_policy_plan.command_routing().map(|policy| {
                crate::runtime::command_routing::UiCommandRoutingRuntimeState::new(
                    crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                    app.capabilities().commands(),
                    policy,
                )
            }),
        );
        let application =
            crate::runtime::session::WorthUiApplicationSessionState::new(app, runtime);
        let pointer_presence_enabled =
            appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Hover);
        let pressed_appearance_enabled =
            appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Pressed);
        let mounted = crate::mounting::WorthUiMountedSessionState::new(
            host_session.identity(),
            mounted_frame_retention_budget,
            presentation_async,
        )
        .map_err(|_| crate::runtime::WorthUiRuntimeLaunchDenial::MountedIdentityExhausted)?;
        let visual_inspection =
            crate::inspection::visual_snapshot::WorthUiVisualInspectionAuthority::seal(
                identity,
                visual_policy,
            );
        Ok(Self {
            identity,
            application,
            host_session,
            mounted,
            host_exchange: crate::host_exchange::WorthUiHostExchangeSessionState::new(
                host_observation_capacity,
            ),
            interaction: crate::runtime::interaction::UiInteractionRuntimeState::new(
                pointer_presence_enabled,
                pressed_appearance_enabled,
                crate::runtime::interaction::UiPointerPresenceCapacity::from_host_observation(
                    host_observation_capacity,
                ),
            ),
            focus: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.focus().map(|policy| {
                    let restoration = service_policy_plan
                        .portal()
                        .is_none_or(crate::declaration::UiPortalPolicy::restores_focus);
                    crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate_with_policy(
                        policy.with_scope_restoration(
                            policy.restores_on_scope_close() && restoration,
                        ),
                    )
                }),
            ),
            portal: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.portal().map(|policy| {
                    crate::runtime::portal::UiPortalRuntimeState::new_with_policy_and_ordinal_issuer(
                        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
                        policy,
                        dormant_portal_stack_ordinal_issuer
                            .take()
                            .expect("active session retains one Portal ordinal issuer"),
                    )
                }),
            ),
            dormant_portal_stack_ordinal_issuer,
            motion: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.motion().map(|policy| {
                    crate::runtime::motion::UiMotionRuntimeState::new_with_policy(
                        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                        policy,
                    )
                }),
            ),
            scroll: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.scroll().map(
                    crate::runtime::scroll::UiScrollRuntimeState::new_session_restore_candidate_with_policy,
                ),
            ),
            selection: crate::runtime::UiRuntimeServiceInstallation::from_optional(
                service_policy_plan.selection().map(
                    crate::runtime::selection::UiSelectionRuntimeState::new_session_restore_candidate_with_policy,
                ),
            ),
            command_routing,
            ime_composing: false,
            portal_exit_retention: super::portal_exit_retention::UiPortalExitRetentionCoordinator::new(),
            intent_evidence: crate::inspection::intent::UiIntentEvidenceRegistry::new(
                identity.as_u64(),
            ),
            intent_application_facts,
            intent_execution: crate::runtime::intent_execution::UiIntentExecutionState::new(),
            intent_admission: crate::runtime::intent::UiIntentAdmissionState::new(
                appearance_axis_demand.contains(worth_ui_dsl::UiAppearanceStateAxis::Operability),
            ),
            intent_confirmation: crate::runtime::intent::UiIntentConfirmationState::new(),
            intent_postures: crate::mounting::UiIntentPostureTable::new(),
            presentation,
            appearance_theme_admission,
            appearance_inspection:
                crate::runtime::appearance::UiAppearanceInspectionProducer::new(initial_generation),
            appearance_owner_snapshot: None,
            visual_inspection,
            next_visual_capture_identity: 1,
            next_visual_overlay_identity: 1,
            next_portal_service_event_identity: 1_u64 << 63,
            visual_captures: crate::inspection::visual_snapshot::UiVisualCaptureRegistry::new(
                visual_policy,
            ),
            visual_overlays: crate::inspection::visual_snapshot::UiVisualOverlayRegistry::new(),
            rebind: crate::runtime::rebind::UiRebindRuntimeState::new(rebind_profile),
        })
    }
}
