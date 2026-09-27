use super::UiNativeViewportBasis;
use crate::certification_support::ScriptedPresentationHost;
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;

#[test]
fn same_scale_readiness_preserves_binding_and_leaves_real_rebind_available() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host)
        .launch_native_surface()
        .expect("native viewport shell should launch");
    let initial_binding = shell.binding;

    shell
        .rebind_native_surface_scale(1_000)
        .expect("equal scale should already be ready");

    assert_eq!(shell.binding, initial_binding);
    assert!(shell.pending_native_surface_reconciliation().is_none());
    shell
        .rebind_native_surface_scale(2_000)
        .expect("equal-scale readiness must leave a real scale successor available");
    assert_ne!(shell.binding, initial_binding);
    assert!(shell.pending_native_surface_reconciliation().is_some());
}

#[test]
fn prepared_frame_uses_the_reconciliation_lane_after_scale_rebind() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native viewport shell should launch");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    host.push_native_display_presented();
    let Ok(crate::mounting::UiMountedFrameOutcome::Published(_)) = shell.present_frame(1, 0) else {
        panic!("baseline frame should publish before scale reconciliation");
    };
    shell
        .rebind_native_surface_scale(2_000)
        .expect("new scale should establish a replacement binding");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    let Ok(frame) = shell.prepare_frame() else {
        panic!("replacement binding should prepare a reconciliation frame");
    };
    host.push_native_display_presented();

    let outcome = shell
        .present_prepared_frame(frame, u64::MAX, 2)
        .expect("prepared reconciliation frame should use its matching lane");

    match outcome {
        crate::mounting::UiMountedFrameOutcome::Reconciled(_)
        | crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {}
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("reconciliation admission denied: {:?}", rejection.denial())
        }
        _ => panic!("prepared reconciliation did not settle"),
    }
    assert!(shell.pending_native_surface_reconciliation().is_none());
}

#[test]
fn same_physical_extent_with_new_scale_and_binding_remeasures_before_projection() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native viewport shell should launch");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    let baseline_calls = host.viewport_measurement_calls();
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    let initial_layout_viewport = shell.native_layout_viewport().unwrap();
    assert_eq!(
        [
            initial_layout_viewport.width(),
            initial_layout_viewport.height()
        ],
        [800.0, 600.0]
    );

    host.set_viewport_extent([400.0, 300.0]);
    shell
        .rebind_native_surface_scale(2_000)
        .expect("scale successor should rebind the native surface");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    shell.observe_native_viewport_readiness([800, 600], 2_000, true);
    let rebound_layout_viewport = shell.native_layout_viewport().unwrap();
    assert_eq!(
        [
            rebound_layout_viewport.width(),
            rebound_layout_viewport.height()
        ],
        [400.0, 300.0]
    );
    assert!(shell.native_viewport_presentation_pending());
    let pending = shell
        .viewport
        .owed()
        .expect("complete basis change must schedule measurement");
    assert_eq!(pending.client_physical_extent, [800, 600]);
    assert_eq!(pending.scale_factor_milli, 2_000);
    assert_eq!(pending.binding, shell.binding);
    host.push_native_display_presented();

    assert!(shell.present_frame(2, 0).is_ok());
    assert_eq!(host.viewport_measurement_calls(), baseline_calls + 1);
    assert_eq!(shell.viewport.owed(), None);
    assert!(!shell.native_viewport_presentation_pending());
    assert_eq!(
        shell.viewport.observed(),
        Some(UiNativeViewportBasis {
            client_physical_extent: [800, 600],
            scale_factor_milli: 2_000,
            binding: shell.binding,
        })
    );
}

#[test]
fn denied_viewport_settlement_retains_exact_basis_and_retries_before_frame_effects() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native viewport shell should launch");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    let baseline_calls = host.viewport_measurement_calls();
    host.push_in_flight(
        vec![crate::certification_support::ScriptedSurfaceCompletion::Pending],
        crate::facade::mounted::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let Ok(crate::facade::mounted::UiMountedFrameOutcome::InFlight(in_flight)) =
        shell.present_frame(1, 0)
    else {
        panic!("predecessor frame must remain in flight")
    };
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    shell.observe_native_viewport_readiness([960, 600], 1_000, true);
    assert!(shell.native_viewport_presentation_pending());
    let pending = shell
        .viewport
        .owed()
        .expect("extent successor must schedule measurement");

    assert!(shell.present_frame(2, 1).is_err());
    assert_eq!(host.presentation_calls(), 1);
    assert_eq!(shell.viewport.owed(), Some(pending));

    let _ = shell.cancel_mounted_presentation(in_flight);
    host.set_viewport_extent([960.0, 600.0]);
    host.push_native_display_presented();
    assert!(shell.present_frame(3, 2).is_ok());
    assert_eq!(host.presentation_calls(), 2);
    assert_eq!(host.viewport_measurement_calls(), baseline_calls + 1);
    assert_eq!(shell.viewport.owed(), None);
    assert!(!shell.native_viewport_presentation_pending());
}

/// A shell whose baseline frame published at 800 physical pixels wide,
/// with the viewport measurements that publication spent.
fn published_at_800_wide() -> (
    ScriptedPresentationHost,
    super::super::WorthUiNativeApplicationShell,
    usize,
) {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native viewport shell should launch");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    host.push_native_display_presented();
    let Ok(crate::mounting::UiMountedFrameOutcome::Published(_)) = shell.present_frame(1, 0) else {
        panic!("baseline frame should publish");
    };
    let baseline_calls = host.viewport_measurement_calls();
    (host, shell, baseline_calls)
}

#[test]
fn a_frame_rejected_before_effects_keeps_its_newer_extent_owed_until_a_retry_presents() {
    use crate::certification_support::ScriptedPresentationOutcome;
    use crate::mounting::UiMountedFrameOutcome;
    use worth_ui_host_contract::UiHostSurfacePresentationDenial;

    let (host, mut shell, baseline_calls) = published_at_800_wide();
    shell.observe_native_viewport_readiness([960, 600], 1_000, true);
    host.set_viewport_extent([960.0, 600.0]);
    let newer = shell.viewport.owed().expect("the newer extent is owed");

    // A terminal rejection ends no debt and owes no host retry.
    host.push_rejected();
    let Ok(UiMountedFrameOutcome::RejectedBeforeEffects(_)) = shell.present_frame(2, 1) else {
        panic!("the adapter declines the frame before effects");
    };
    assert_eq!(shell.viewport.owed(), Some(newer));
    assert!(!shell.native_presentation_retry_pending());

    // A timeout rejection owes the successor at the host's retry wake.
    host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    let Ok(UiMountedFrameOutcome::RejectedBeforeEffects(_)) = shell.present_frame(3, 2) else {
        panic!("the host times out before effects");
    };
    assert_eq!(shell.viewport.owed(), Some(newer));
    assert!(shell.native_viewport_presentation_pending());
    assert!(shell.native_presentation_retry_pending());

    host.push_native_display_as_issued();
    let Ok(UiMountedFrameOutcome::Published(_)) = shell.present_frame(4, 3) else {
        panic!("the retry should publish the newer extent");
    };
    assert_eq!(shell.viewport.owed(), None);
    assert!(!shell.native_presentation_retry_pending());
    assert_eq!(host.presentation_calls(), 4);
    assert_eq!(host.viewport_measurement_calls(), baseline_calls + 1);
    assert_eq!(
        shell
            .native_layout_viewport()
            .map(|viewport| viewport.width()),
        Some(960.0)
    );
}

#[test]
fn a_delayed_completion_lands_only_the_extent_its_frame_measured() {
    use crate::certification_support::{
        ScriptedPresentationAcknowledgement, ScriptedSurfaceCompletion,
    };
    use crate::mounting::UiMountedFrameOutcome;

    let (host, mut shell, baseline_calls) = published_at_800_wide();
    shell.observe_native_viewport_readiness([900, 600], 1_000, true);
    host.set_viewport_extent([900.0, 600.0]);
    host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(2),
                // This fixture's content is width-independent, so the
                // wider frame settles without paint.
                crate::facade::mounted::UiMountedCompletedEffects::new(Vec::new()),
                crate::facade::mounted::UiHostPresentationCostReport::default(),
            )),
        ],
        crate::facade::mounted::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let Ok(UiMountedFrameOutcome::InFlight(mut in_flight)) = shell.present_frame(1_000, 1) else {
        panic!("the 900 frame should stay in flight");
    };

    shell.observe_native_viewport_readiness([960, 600], 1_000, true);
    host.set_viewport_extent([960.0, 600.0]);
    let newer = shell.viewport.owed().expect("the newer extent is owed");
    let completed = loop {
        match shell.complete_frame_presentation(in_flight, 2) {
            UiMountedFrameOutcome::InFlight(pending) => in_flight = pending,
            outcome => break outcome,
        }
    };
    assert!(matches!(completed, UiMountedFrameOutcome::Published(_)));
    assert_eq!(shell.viewport.owed(), Some(newer));
    assert_eq!(host.viewport_measurement_calls(), baseline_calls + 1);

    host.push_native_display_as_issued();
    let Ok(UiMountedFrameOutcome::Published(_)) = shell.present_frame(3, 2) else {
        panic!("the newer extent should publish");
    };
    assert_eq!(shell.viewport.owed(), None);
    assert_eq!(host.viewport_measurement_calls(), baseline_calls + 2);
    assert_eq!(
        shell
            .native_layout_viewport()
            .map(|viewport| viewport.width()),
        Some(960.0)
    );
}
