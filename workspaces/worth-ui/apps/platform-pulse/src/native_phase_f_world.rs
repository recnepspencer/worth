use std::process::ExitCode;

use worth_ui_native_platform::{
    UiNativeClientPresentationTransitionKind as Kind,
    UiNativeClientPresentationTransitionObservation as Transition, UiNativePlatformOutcome,
    UiNativePlatformProfile, UiNativeQualificationPlan, UiNativeWindowSpec, WorthUiNativePlatform,
};

pub(crate) fn run() -> ExitCode {
    let qualification = UiNativeQualificationPlan::effects_indeterminate_on_presentation(3)
        .and_then(|plan| plan.with_deferred_completion_on_presentation(1))
        .and_then(|plan| plan.with_deferred_completion_on_presentation(2))
        .and_then(|plan| plan.with_duplicate_completion_observation_on_presentation(2))
        .expect("the Phase F physical observation ordinals are bounded");
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Phase F Async Courtroom",
        [160, 96],
    ))
    .with_native_qualification_plan(qualification);
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let Some(presentation_async) =
        crate::query_source::install_native_presentation_async_for_transition_courtroom()
    else {
        return ExitCode::from(2);
    };
    let application = crate::native_phase_f_application::PlatformPulseNativePhaseFApplication::new(
        presentation_async,
    );
    let application =
        if std::env::args_os().any(|argument| argument == "--worth-ui-native-external-close") {
            application.remain_open_until_external_close()
        } else {
            application
        };
    let outcome = platform.run(application);
    let UiNativePlatformOutcome::Closed(receipt) = outcome else {
        eprintln!("worth-ui-native-phase-f stopped: {outcome:?}");
        return ExitCode::from(3);
    };
    adjudicate(receipt)
}

fn adjudicate(receipt: worth_ui_native_platform::UiNativePlatformCloseReceipt) -> ExitCode {
    let Some(shutdown) = receipt.client_shutdown() else {
        return ExitCode::from(3);
    };
    let presentation = receipt.presentation();
    let attribution = receipt.client_attribution();
    crate::native_phase_f_world_evidence::publish(&receipt, shutdown);
    if trace_matches_contract(shutdown.presentation_transitions())
        && shutdown.presentation_transition_trace_complete()
        && !shutdown.text_presentation_work().is_empty()
        && shutdown.text_presentation_work_trace_complete()
        && !receipt.text_atlas_plan_observations().is_empty()
        && !receipt.physical_signal_transition_observations().is_empty()
        && receipt.physical_signal_transition_trace_complete()
        && shutdown.managed_semantic_resources_complete()
        && shutdown.managed_semantic_resources_closed() > 0
        && presentation.presented_frame() == attribution.frame()
        && presentation.binding_generation() == attribution.binding()
        && presentation.presentation_attempt() == attribution.presentation_attempt()
        && !presentation.intrinsic_glyphs().is_empty()
        && receipt.retained_frames().iter().any(|frame| {
            frame.kind() == worth_ui_native_platform::UiNativePresentationWorkKind::Reconstruction
        })
        && receipt.observation_history_complete()
        && receipt.terminal_census().is_zero()
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

fn trace_matches_contract(transitions: &[Transition]) -> bool {
    let [a_pending, b_supersedes, a_stale, b_completed, b_duplicate, c_pending, c_unresolved, c_recovery, reconstruction, terminal] =
        transitions
    else {
        return false;
    };
    let expected = [
        Kind::Pending,
        Kind::Superseded,
        Kind::StaleCompletionRejected,
        Kind::Completed,
        Kind::DuplicateCompletionRejected,
        Kind::Pending,
        Kind::Unresolved,
        Kind::RecoveryRequired,
        Kind::ReconstructionCurrent,
        Kind::TerminalClosed,
    ];
    transitions
        .iter()
        .zip(expected)
        .all(|(observed, expected)| observed.kind() == expected)
        && same_request(*a_pending, *a_stale)
        && !same_request(*a_pending, *b_supersedes)
        && a_pending.binding() == b_supersedes.binding()
        && same_request(*b_supersedes, *b_completed)
        && same_request(*b_completed, *b_duplicate)
        && same_request(*c_pending, *c_unresolved)
        && same_request(*c_unresolved, *c_recovery)
        && !same_request(*c_recovery, *reconstruction)
        && c_recovery.binding() != reconstruction.binding()
        && same_request(*reconstruction, *terminal)
}

fn same_request(left: Transition, right: Transition) -> bool {
    left.attempt() == right.attempt() && left.binding() == right.binding()
}
