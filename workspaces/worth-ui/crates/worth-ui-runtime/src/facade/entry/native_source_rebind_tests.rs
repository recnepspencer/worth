use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;
use crate::runtime::{WorthUiReloadDebounce, WorthUiSourceProvider, WorthUiWatcherEvent};

use super::{WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial};

#[test]
fn managed_source_rebind_remains_owned_until_host_progress_or_shutdown() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("source-backed native fixture should launch");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));

    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let first_request = source_request(&shell, 2);
    let outcome = shell
        .begin_managed_source_rebind(first_request)
        .expect("source successor should reach managed presentation");
    assert!(matches!(
        outcome,
        WorthUiNativeManagedSourceRebindOutcome::Pending
    ));
    assert_eq!(host.native_in_flight_count(), 1);

    let second_request = source_request(&shell, 3);
    let denial = match shell.begin_managed_source_rebind(second_request) {
        Err(denial) => denial,
        Ok(_) => panic!("pending source work must retain the sole managed slot"),
    };
    assert!(matches!(
        denial,
        WorthUiNativeSourceRebindDenial::ManagedRebindAlreadyInFlight
    ));
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(host.native_in_flight_count(), 0);
}

fn source_request(
    shell: &super::WorthUiNativeApplicationShell,
    tick: u64,
) -> crate::runtime::rebind::UiSourceRebindRequest {
    let provider = WorthUiSourceProvider::in_memory(format!("managed-source-{tick}")).with_file(
        "app/main.wui",
        "component workspace.component.active_session_candidate { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }",
    );
    let events = [WorthUiWatcherEvent::provider_revision(provider.id())];
    let snapshot = WorthUiReloadDebounce::default()
        .debounce(provider, &events, tick)
        .expect("complete in-memory source should settle");
    crate::runtime::rebind::UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(tick.saturating_add(10)))
        .observed_at_tick(tick)
}
