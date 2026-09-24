use super::*;
use worth_ui_host_contract::{UiHostSurfacePresentationDenial, UiMountedRgba8};
use worth_ui_runtime::certification_support::ScriptedPresentationOutcome;

#[test]
fn programmatic_theme_first_publication_rejection_retry_and_switch_back() {
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
    assert_eq!(
        host.last_surface_colors(),
        vec![UiMountedRgba8::new(24, 48, 160, 255)]
    );
    let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => {
            frame.presentation().surfaces()[0].semantic_surface()
        }
        _ => panic!("the first publication owns the surface"),
    };
    let blue = shell.active_theme_binding(surface).unwrap().clone();
    let calls = host.presentation_calls();
    let green = shell
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
        )
        .unwrap();
    let request = shell
        .prepare_programmatic_theme_switch(surface, blue.binding_generation(), green)
        .unwrap();
    assert_eq!(
        host.presentation_calls(),
        calls,
        "observing the request must not publish an unrelated posture frame"
    );
    host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
    ));
    {
        let outcome = shell
            .begin_theme_switch(
                request,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(2),
            )
            .unwrap();
        assert!(matches!(
            outcome,
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        ));
        host.push_native_display_presented();
        assert!(matches!(
            shell.retry_managed_rebind(3),
            Ok(WorthUiNativeManagedRebindProgress::Published(_))
        ));
    }
    assert_eq!(
        host.last_surface_colors(),
        vec![UiMountedRgba8::new(16, 144, 48, 255)],
        "the first accepted successor carries Green work"
    );
    assert_eq!(
        shell
            .active_theme_binding(surface)
            .unwrap()
            .binding_generation(),
        blue.binding_generation() + 1
    );

    let current = shell
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let capability = shell
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new("theme.command.blue").unwrap(),
        )
        .unwrap();
    let request = shell
        .prepare_programmatic_theme_switch(surface, current, capability)
        .unwrap();
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .begin_theme_switch(
                request,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(4)
            )
            .unwrap(),
        WorthUiNativeManagedRebindProgress::Published(_)
    ));
    assert_eq!(
        host.last_surface_colors(),
        vec![UiMountedRgba8::new(24, 48, 160, 255)]
    );
    assert_eq!(host.pending_presentation_count(), 0);
}

#[test]
fn intervening_owner_turn_denies_programmatic_origin_before_presentation() {
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
    let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => {
            frame.presentation().surfaces()[0].semantic_surface()
        }
        _ => panic!("accepted surface required"),
    };
    let binding = shell
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let capability = shell
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
        )
        .unwrap();
    let stale = shell
        .prepare_programmatic_theme_switch(surface, binding, capability.clone())
        .unwrap();
    let current = shell
        .prepare_programmatic_theme_switch(surface, binding, capability)
        .unwrap();
    let calls = host.presentation_calls();
    assert!(matches!(
        shell.begin_theme_switch(
            stale,
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(2)
        ),
        Err(UiNativeThemeSwitchDenial::Admission(
            UiThemeSwitchPreparationDenial::Origin(
                UiThemeSwitchOriginAdmissionDenial::ObservationNotClosed
            )
        ))
    ));
    assert_eq!(host.presentation_calls(), calls);
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .begin_theme_switch(
                current,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(3)
            )
            .unwrap(),
        WorthUiNativeManagedRebindProgress::Published(_)
    ));
    assert_eq!(
        host.last_surface_colors(),
        vec![UiMountedRgba8::new(16, 144, 48, 255)]
    );
}
