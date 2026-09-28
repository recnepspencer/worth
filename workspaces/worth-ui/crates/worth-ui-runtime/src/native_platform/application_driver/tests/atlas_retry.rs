use super::retryable_program;

/// A frame the driver completes asynchronously on a replacement binding the
/// shell still owes, deferred by the host's text atlas, retries as that
/// reconciliation. Admission refuses an ordinary retry while the binding it
/// replaces is blocked, which failed the driver mid-resize. The frame settles
/// as ordinary program progress; a driver reconstruction the host required
/// reconstructs again instead, and one under physical recovery authority is
/// not covered here.
#[test]
fn an_atlas_deferred_completion_on_an_owed_binding_retries_as_its_reconciliation() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome};

    let (host, mut shell, mut progress) = retryable_program();
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    host.push_native_display_presented();
    assert!(matches!(
        shell.present_frame(1, 0),
        Ok(UiMountedFrameOutcome::Published(_))
    ));
    shell.observe_native_viewport_readiness([820, 600], 1_000, true);
    host.set_viewport_extent([820.0, 600.0]);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let outcome = shell
        .reconstruct_native_surface_successor(u64::MAX, 1)
        .expect("the reconstruction prepares");
    assert!(matches!(outcome, UiMountedFrameOutcome::InFlight(_)));
    progress
        .retain_or_attribute(
            &mut shell,
            outcome,
            super::super::program_progress::UiNativePresentationSource::Program(0),
            None,
            None,
            false,
        )
        .expect("the driver retains the in-flight reconstruction");
    assert_eq!(progress.pending.len(), 1);
    assert!(
        shell.rebind_native_surface_scale(2_000).is_err(),
        "the replacement binding is still owed"
    );

    host.push_native_display_as_issued();
    // The program's one frame is behind it, so settling presents nothing more.
    progress.next_frame = 1;
    progress
        .settle_first_pending_presentation_for_test(&mut shell)
        .expect("the atlas-deferred completion retries as the owed reconciliation");
    assert_eq!(host.presentation_calls(), 3);
    assert!(
        shell.rebind_native_surface_scale(2_000).is_ok(),
        "the retry's publication proves the replacement"
    );
}
