use super::*;
use worth_ui::facade::app::{
    WorthUiNativeInteractionIngressStop, WorthUiNativeManagedRebindProgress,
};
use worth_ui_host_contract::{UiHostSurfacePresentationDenial, UiHostSurfacePresentationOutcome};

#[test]
fn pending_posture_retains_native_input_until_physical_retry_settles() {
    let host = native_command_host();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut shell = world::application(host.clone())
        .launch_native_surface()
        .unwrap();
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    host.push_native_display_presented();
    assert!(matches!(
        shell.present_frame(10, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));
    let definition = || {
        UiIntentDefinition::<AdvanceStatus>::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        )
    };
    let ingress = shell.admit_native_intent_observations(
        definition(),
        shortcut_drain(
            shell.host_session_identity().as_u64(),
            current_presentation(&shell),
            false,
        ),
        crate::intent::execution::execution_deadline(20),
    );
    let mut transitions = ingress.into_transitions().into_vec();
    assert_eq!(transitions.len(), 1);
    let WorthUiNativeIntentTransition::AttemptPrepared(prepared) = transitions.remove(0) else {
        panic!("shortcut must admit the real command");
    };
    host.push_presentation(UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
    ));
    assert!(matches!(
        shell.begin_managed_native_intent_posture_publication(prepared.into_posture(), 21),
        Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Pending)
    ));
    let predecessor = current_presentation(&shell);
    let calls = host.presentation_calls();
    let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => {
            frame.presentation().surfaces()[0].semantic_surface()
        }
        _ => panic!("pending posture preserves the accepted surface"),
    };
    let theme_predecessor = shell
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let green = shell
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
        )
        .unwrap();
    assert!(matches!(
        shell.prepare_programmatic_theme_switch(surface, theme_predecessor, green.clone()),
        Err(UiProgrammaticThemeSwitchPreparationDenial::ManagedRebindAlreadyInFlight)
    ));
    assert_eq!(host.presentation_calls(), calls);
    let deferred = shell.admit_native_intent_observations(
        definition(),
        shortcut_drain_at(
            shell.host_session_identity().as_u64(),
            predecessor,
            false,
            2,
        ),
        crate::intent::execution::execution_deadline(22),
    );
    assert!(deferred.transitions().is_empty());
    let mut stops = deferred.into_interaction_stops().into_vec();
    assert_eq!(stops.len(), 1);
    let WorthUiNativeInteractionIngressStop::ManagedPublicationPending(drain) = stops.remove(0)
    else {
        panic!("pending publication must return ownership of the input drain");
    };
    assert_eq!(current_presentation(&shell), predecessor);
    assert_eq!(host.presentation_calls(), calls);
    host.push_native_display_settled_without_effects();
    let progress =
        worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas,
                None,
                false,
            ),
        );
    let receipt = match shell.progress_managed_rebind(&progress).unwrap() {
        WorthUiNativeManagedRebindProgress::Published(receipt) => receipt,
        _ => panic!("physical text progress must settle the exact pending posture"),
    };
    assert!(
        shell
            .issue_theme_switch_origin_from_publication(&receipt)
            .is_ok(),
        "the accepted posture commits the same closed owner turn that prepared its pixels"
    );
    drop(receipt);
    assert!(shell
        .prepare_programmatic_theme_switch(surface, theme_predecessor, green)
        .is_ok());
    // Reuse the exact move-only drain. Its old presentation may now be stale;
    // that is an admission decision after settlement, not permission to lose it.
    let resumed = shell.admit_native_intent_observations(
        definition(),
        drain,
        crate::intent::execution::execution_deadline(23),
    );
    assert!(
        !resumed.into_interaction_stops().iter().any(|stop| matches!(
            stop,
            WorthUiNativeInteractionIngressStop::ManagedPublicationPending(_)
        ))
    );
    // This posture does not change the role's paint. The accepted retry has
    // no color delta; last_surface_colors records work, not retained pixels.
    assert!(host.last_surface_colors().is_empty());
}
