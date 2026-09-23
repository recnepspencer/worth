use std::process::ExitCode;

use worth_ui_native_platform::{
    UiNativeClientPresentationTransitionKind as Kind,
    UiNativeClientPresentationTransitionObservation as Transition,
    UiNativePhysicalSignalExternalStatusClass as ExternalStatus,
    UiNativePhysicalSignalObservationOriginClass as Origin,
    UiNativePhysicalSignalSettlementClass as Settlement,
    UiNativePhysicalSignalTransitionObservation as PhysicalTransition,
    UiNativePhysicalSignalWorkClass as Work, UiNativePlatformOutcome, UiNativePlatformProfile,
    UiNativeQualificationPlan, UiNativeWindowSpec, WorthUiNativePlatform,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT;

pub(crate) fn run() -> ExitCode {
    let qualification = UiNativeQualificationPlan::deferred_completion_on_presentation(2)
        .expect("the Phase F cancellation presentation ordinal is bounded");
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Phase F Partial Effects Cancellation Courtroom",
        PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT,
    ))
    .with_native_qualification_plan(qualification);
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let Some(presentation_async) = crate::query_source::install_native_presentation_async() else {
        return ExitCode::from(2);
    };
    let outcome = platform.run(
        crate::native_phase_f_application::PlatformPulseNativePhaseFApplication::new(
            presentation_async,
        )
        .cancel_after_external_submission(),
    );
    let UiNativePlatformOutcome::Closed(receipt) = outcome else {
        eprintln!("worth-ui-native-phase-f cancellation stopped: {outcome:?}");
        return ExitCode::from(3);
    };
    let Some(shutdown) = receipt.client_shutdown() else {
        return ExitCode::from(3);
    };
    let transitions = shutdown.presentation_transitions();
    let lifecycle = receipt.physical_signal_lifecycle();
    let exact_query_recovery = trace_matches_contract(transitions);
    let physical_transitions = receipt
        .physical_signal_transition_observations()
        .iter()
        .copied()
        .map(physical_transition_evidence)
        .collect::<Vec<_>>();
    let exact_cancelled_physical_request = transitions.get(2).is_some_and(|cancelled| {
        cancellation_follows_exact_external_submission(
            receipt.physical_signal_transition_observations(),
            *cancelled,
        )
    });
    let exact_physical_owner = lifecycle.cancellations() == 1
        && lifecycle.recovery_schedules() == 1
        && lifecycle.recovery_resolutions() == 1;
    let evidence = serde_json::json!({
        "schema": "worth-ui-native-phase-f-partial-cancellation-world-v1",
        "presentation_transitions": transitions.iter().map(|transition| serde_json::json!({
            "kind": format!("{:?}", transition.kind()),
            "attempt": transition.attempt(),
            "binding": transition.binding(),
        })).collect::<Vec<_>>(),
        "exact_query_recovery": exact_query_recovery,
        "exact_cancelled_physical_request": exact_cancelled_physical_request,
        "physical_signal": {
            "cancellations": lifecycle.cancellations(),
            "recovery_schedules": lifecycle.recovery_schedules(),
            "recovery_resolutions": lifecycle.recovery_resolutions(),
            "transition_trace_complete": receipt.physical_signal_transition_trace_complete(),
            "transitions": physical_transitions,
        },
        "terminal_zero": receipt.terminal_census().is_zero(),
        "query_close_complete": shutdown.managed_semantic_resources_complete(),
    });
    println!("{evidence}");
    if exact_query_recovery
        && exact_physical_owner
        && exact_cancelled_physical_request
        && receipt.physical_signal_transition_trace_complete()
        && receipt.terminal_census().is_zero()
        && shutdown.managed_semantic_resources_complete()
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

fn cancellation_follows_exact_external_submission(
    physical: &[PhysicalTransition],
    cancelled: Transition,
) -> bool {
    let exact = |row: &PhysicalTransition| {
        row.work() == Work::Presentation
            && row.attempt() == cancelled.attempt()
            && row.binding() == cancelled.binding()
            && row.host_session() > 0
            && row.surface() > 0
            && row.host_surface() > 0
            && row.request_sequence() > 0
    };
    let pending = physical
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            exact(row)
                && row.origin() == Origin::NativeExternalPort
                && row.external_status() == ExternalStatus::Pending
                && row.settlement() == Settlement::Pending
        })
        .collect::<Vec<_>>();
    let cancellation = physical
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            exact(row)
                && row.origin() == Origin::PhysicalOwnerCancellation
                && row.external_status() == ExternalStatus::CancellationEffectsMayHaveBegun
                && row.settlement() == Settlement::Indeterminate
        })
        .collect::<Vec<_>>();
    let ([(pending_index, pending)], [(cancellation_index, cancellation)]) =
        (pending.as_slice(), cancellation.as_slice())
    else {
        return false;
    };
    pending_index < cancellation_index
        && pending.host_session() == cancellation.host_session()
        && pending.surface() == cancellation.surface()
        && pending.host_surface() == cancellation.host_surface()
        && pending.request_sequence() == cancellation.request_sequence()
}

fn physical_transition_evidence(
    transition: worth_ui_native_platform::UiNativePhysicalSignalTransitionObservation,
) -> serde_json::Value {
    serde_json::json!({
        "host_session": transition.host_session(),
        "attempt": transition.attempt(),
        "surface": transition.surface(),
        "host_surface": transition.host_surface(),
        "binding": transition.binding(),
        "request_sequence": transition.request_sequence(),
        "work": format!("{:?}", transition.work()),
        "origin": format!("{:?}", transition.origin()),
        "external_status": format!("{:?}", transition.external_status()),
        "settlement": format!("{:?}", transition.settlement()),
        "performed_transitions": transition.performed_transitions(),
        "performed_nodes": transition.performed_nodes(),
    })
}

fn trace_matches_contract(transitions: &[Transition]) -> bool {
    let [initial_pending, initial_completed, cancelled_pending, unresolved, recovery, reconstruction, terminal] =
        transitions
    else {
        return false;
    };
    let expected = [
        Kind::Pending,
        Kind::Completed,
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
        && same_request(*initial_pending, *initial_completed)
        && same_request(*cancelled_pending, *unresolved)
        && same_request(*unresolved, *recovery)
        && !same_request(*recovery, *reconstruction)
        && recovery.binding() != reconstruction.binding()
        && same_request(*reconstruction, *terminal)
}

fn same_request(left: Transition, right: Transition) -> bool {
    left.attempt() == right.attempt() && left.binding() == right.binding()
}
