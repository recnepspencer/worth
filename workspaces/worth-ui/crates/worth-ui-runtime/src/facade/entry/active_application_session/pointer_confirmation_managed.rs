use crate::certification_support::ScriptedPresentationAcknowledgement;
use worth_ui_host_contract::*;

pub(super) fn journey(
    mut shell: crate::facade::WorthUiNativeApplicationShell,
    host: crate::certification_support::ScriptedPresentationHost,
    expires: u64,
) {
    shell
        .install_native_observation_clock(
            worth_ui_host_native::UiNativeObservationClock::from_certification_elapsed(3).unwrap(),
        )
        .unwrap();
    let provider =
        crate::runtime::WorthUiSourceProvider::rust_authored("managed-confirmation-successor")
            .with_rust_authored_input(super::fixture::source());
    let events = [crate::runtime::WorthUiWatcherEvent::provider_revision(
        provider.id(),
    )];
    let snapshot = crate::runtime::WorthUiReloadDebounce::default()
        .debounce(provider, &events, 1)
        .unwrap();
    host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Presented(
                ScriptedPresentationAcknowledgement::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    crate::certification_support::scripted_presentation_epoch(),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    assert!(matches!(
        shell
            .begin_managed_source_rebind(
                crate::runtime::rebind::UiSourceRebindRequest::new(snapshot).observed_at_tick(4),
            )
            .unwrap(),
        crate::facade::entry::WorthUiNativeManagedSourceRebindOutcome::Pending
    ));
    let predecessor = shell.session.active_generation_identity();
    let observation = shell
        .session
        .pointer_affordance_snapshot
        .as_ref()
        .unwrap()
        .observation_identity();
    let calls = host.presentation_calls();
    // Only the host clock epoch is simulated. The candidate and managed completion
    // remain owned by their production entry points throughout the elapsed interval.
    shell.session.observation_clock = Some(
        worth_ui_host_native::UiNativeObservationClock::from_certification_elapsed(expires + 1)
            .unwrap(),
    );
    assert_eq!(shell.close_native_observation_time(), Ok(Some(expires + 1)));
    assert_eq!(
        shell
            .session
            .pointer_affordance_snapshot
            .as_ref()
            .unwrap()
            .observation_identity(),
        observation
    );
    assert_eq!(host.presentation_calls(), calls);
    let progress = crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(
        worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
            None,
            false,
        ),
    );
    assert!(matches!(
        shell.progress_managed_rebind(&progress).unwrap(),
        crate::facade::entry::WorthUiNativeManagedRebindProgress::Published(_)
    ));
    assert_ne!(shell.session.active_generation_identity(), predecessor);
    assert_eq!(shell.close_native_observation_time(), Ok(None));
    assert!(
        !shell.native_pointer_presentation_pending(),
        "first accepted successor already excludes the predecessor confirmation authority"
    );
    assert_eq!(host.presentation_calls(), calls);
    let surface =
        shell.session.inspect_mounted_identity().surface_bindings()[0].semantic_surface_identity();
    assert_eq!(
        shell
            .session
            .mounted
            .current_pointer_affordance_for_test(surface)
            .unwrap()
            .family(),
        UiPointerAffordanceFamily::Default
    );
    let _ = shell.shutdown();
    assert_eq!(host.native_in_flight_count(), 0);
}
