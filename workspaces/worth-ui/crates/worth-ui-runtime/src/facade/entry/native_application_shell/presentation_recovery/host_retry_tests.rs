use super::super::super::WorthUiNativeApplicationShell;
use crate::certification_support::{ScriptedPresentationHost, ScriptedPresentationOutcome};
use crate::facade::mounted::{UiHostSurfacePresentationDenial, UiMountedFrameOutcome};

/// A shell whose host timed out its 960x600 frame before effects, with the
/// wake owed to that frame's successor.
fn timed_out_shell() -> (ScriptedPresentationHost, WorthUiNativeApplicationShell) {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = crate::runtime::tests::active_application_session_test_support::
        source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native viewport shell launches");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    host.push_native_display_presented();
    let Ok(UiMountedFrameOutcome::Published(_)) = shell.present_frame(1, 0) else {
        panic!("baseline frame publishes");
    };
    shell.observe_native_viewport_readiness([960, 600], 1_000, true);
    host.set_viewport_extent([960.0, 600.0]);
    host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    let Ok(UiMountedFrameOutcome::RejectedBeforeEffects(_)) = shell.present_frame(2, 1) else {
        panic!("the host times the frame out before effects");
    };
    assert!(shell.native_presentation_retry_pending());
    (host, shell)
}

/// Motion, managed rebinds, and intent completions present through the
/// session without landing on the shell.
fn present_for_another_owner(shell: &mut WorthUiNativeApplicationShell) -> UiMountedFrameOutcome {
    shell
        .session
        .execute_mounted_frame_with_application_presentation(
            worth_ui_host_contract::UiPresentationDeadline::at_tick(3),
            2,
        )
        .unwrap_or_else(|_| panic!("another owner's frame reaches the host"))
}

#[test]
fn any_presentation_that_begins_effects_ends_the_host_retry() {
    let (host, mut shell) = timed_out_shell();
    host.push_native_display_as_issued();
    assert!(matches!(
        present_for_another_owner(&mut shell),
        UiMountedFrameOutcome::Published(_)
    ));
    assert!(!shell.native_presentation_retry_pending());
    assert!(
        shell.native_viewport_presentation_pending(),
        "the shell's own frame at 960x600 is still owed"
    );
}

#[test]
fn every_host_answer_but_timeout_or_occlusion_ends_the_host_retry() {
    let ending = [
        ScriptedPresentationOutcome::PresentationIndeterminate,
        ScriptedPresentationOutcome::RejectedBeforeEffects(
            UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
        ),
        ScriptedPresentationOutcome::RejectedBeforeEffects(
            UiHostSurfacePresentationDenial::ReconstructionRequired,
        ),
        ScriptedPresentationOutcome::RejectedBeforeEffects(
            UiHostSurfacePresentationDenial::CapacityExceeded,
        ),
    ];
    for answer in ending {
        let indeterminate = matches!(
            answer,
            ScriptedPresentationOutcome::PresentationIndeterminate
        );
        let (host, mut shell) = timed_out_shell();
        host.push_presentation(answer);
        let outcome = present_for_another_owner(&mut shell);
        assert!(
            match outcome {
                UiMountedFrameOutcome::PresentationIndeterminate(_) => indeterminate,
                UiMountedFrameOutcome::RejectedBeforeEffects(_) => !indeterminate,
                _ => false,
            },
            "the scripted answer reaches the session"
        );
        assert!(!shell.native_presentation_retry_pending());
        assert!(shell.native_viewport_presentation_pending());
    }

    for denial in [
        UiHostSurfacePresentationDenial::ExternalTimeout,
        UiHostSurfacePresentationDenial::SurfaceOccluded,
    ] {
        let (host, mut shell) = timed_out_shell();
        host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(denial));
        assert!(matches!(
            present_for_another_owner(&mut shell),
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
        ));
        assert!(
            shell.native_presentation_retry_pending(),
            "the host still holds a wake after {denial:?}"
        );
    }
}

/// A live resize reconstructs onto a replacement binding while the published
/// one waits to be reconciled. When the host defers that frame's text atlas,
/// its retry must still be the reconciliation, or admission refuses it.
#[test]
fn an_atlas_deferred_reconstruction_retries_as_its_reconciliation() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::mounted::UiHostSurfaceCancellationOutcome;

    for asynchronous in [false, true] {
        let host = ScriptedPresentationHost::native_display();
        let mut shell = crate::runtime::tests::active_application_session_test_support::
            source_backed_component_app_with_host_and_viewport_allocation(host.clone())
            .launch_native_surface()
            .expect("native viewport shell launches");
        crate::facade::entry::native_application_identity_trace_test_support::
            install_bound_surface_geometry(&mut shell);
        shell.observe_native_viewport_readiness([800, 600], 1_000, false);
        host.push_native_display_presented();
        let Ok(UiMountedFrameOutcome::Published(_)) = shell.present_frame(1, 0) else {
            panic!("baseline frame publishes");
        };
        shell.observe_native_viewport_readiness([820, 600], 1_000, true);
        host.set_viewport_extent([820.0, 600.0]);
        let deferred = UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred;
        if asynchronous {
            host.push_in_flight(
                vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(deferred)],
                UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
            );
        } else {
            host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(deferred));
        }
        let outcome = shell
            .reconstruct_native_surface_successor(10_000, 1)
            .expect("the reconstruction prepares");
        let outcome = match outcome {
            UiMountedFrameOutcome::InFlight(pending) if asynchronous => {
                shell.complete_frame_presentation(pending, 250)
            }
            outcome => outcome,
        };
        assert!(matches!(
            outcome,
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
        ));
        assert!(shell.pending_native_surface_reconciliation().is_some());

        host.push_native_display_as_issued();
        match shell
            .resume_frame_presentation(outcome, 10_001, 250)
            .expect("the deferred reconstruction resumes")
        {
            UiMountedFrameOutcome::Reconciled(_) => {}
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("retry admission: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
                panic!(
                    "retry rejection ({asynchronous}): {:?}",
                    rejected.rejections()
                )
            }
            other => panic!("retry outcome: {:?}", std::mem::discriminant(&other)),
        }
        assert!(shell.pending_native_surface_reconciliation().is_none());
        assert!(!shell.native_viewport_presentation_pending());
    }
}
