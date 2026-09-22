use std::process::ExitCode;

use worth_ui_native_platform::{
    UiNativeClientPresentationTransitionKind as Kind,
    UiNativeClientPresentationTransitionObservation as Transition, UiNativePlatformOutcome,
    UiNativePlatformProfile, UiNativeQualificationPlan, UiNativeWindowSpec, WorthUiNativePlatform,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT;

pub(crate) fn run() -> ExitCode {
    let qualification = UiNativeQualificationPlan::deferred_completion_on_presentation(1)
        .expect("the Phase F deferred presentation ordinal is bounded");
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Phase F Deferred Completion Courtroom",
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
        .single_current_frame(),
    );
    let UiNativePlatformOutcome::Closed(receipt) = outcome else {
        eprintln!("worth-ui-native-phase-f deferred completion stopped: {outcome:?}");
        return ExitCode::from(3);
    };
    adjudicate(receipt)
}

fn adjudicate(receipt: worth_ui_native_platform::UiNativePlatformCloseReceipt) -> ExitCode {
    let Some(shutdown) = receipt.client_shutdown() else {
        return ExitCode::from(3);
    };
    let transitions = shutdown.presentation_transitions();
    let completed_deferred_request = transitions.windows(2).any(|pair| {
        pair[0].kind() == Kind::Pending
            && pair[1].kind() == Kind::Completed
            && same_request(pair[0], pair[1])
    });
    let terminal_without_recovery = transitions.iter().all(|transition| {
        !matches!(
            transition.kind(),
            Kind::Unresolved | Kind::RecoveryRequired | Kind::ReconstructionCurrent
        )
    });
    let Some(presentation) = receipt.presentation() else {
        return ExitCode::from(3);
    };
    let retained_payload_exact = receipt.retained_frames().last().is_some_and(|retained| {
        retained.frame() == presentation.presented_frame()
            && retained.retained_baseline_rgba8() == presentation.retained_baseline_rgba8()
            && retained.retained_center_rgba8() == presentation.retained_center_rgba8()
            && retained.cost() == presentation.cost()
    });
    let physical_identity_exact = transitions
        .iter()
        .find(|transition| transition.kind() == Kind::Completed)
        .is_some_and(|transition| {
            transition.attempt() == presentation.presentation_attempt()
                && transition.binding() == presentation.binding_generation()
        });
    let cost = presentation.cost();
    let [physical_width, physical_height] = presentation.client_physical_size();
    let presented_pixels = u64::from(physical_width) * u64::from(physical_height);
    let exact_async_cost = cost.presented_surfaces() == 1
        && cost.asynchronous_handoffs() == 1
        && cost.native_resource_cache_misses() == 1
        && cost.cleared_pixels() == presented_pixels
        && cost.presented_pixels() == presented_pixels
        && cost.gpu_writes() == 1
        && cost.render_passes() == 2
        && cost.surface_acquisitions() == 1
        && cost.queue_submissions() == 1
        && cost.presents() == 1;
    let exact_pixels = presentation.source_rgba8() == [255, 255, 255, 255]
        && presentation.retained_baseline_rgba8() == [23, 32, 42, 255];
    println!(
        "{}",
        evidence(
            &receipt,
            transitions,
            DeferredCompletionAdjudication {
                completed_deferred_request,
                terminal_without_recovery,
                retained_payload_exact,
                physical_identity_exact,
                exact_async_cost,
            },
        )
    );
    if completed_deferred_request
        && terminal_without_recovery
        && retained_payload_exact
        && physical_identity_exact
        && exact_async_cost
        && exact_pixels
        && presentation.port_crossings() == 2
        && receipt.observation_history_complete()
        && shutdown.managed_semantic_resources_complete()
        && receipt.terminal_census().is_zero()
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

#[derive(Clone, Copy)]
struct DeferredCompletionAdjudication {
    completed_deferred_request: bool,
    terminal_without_recovery: bool,
    retained_payload_exact: bool,
    physical_identity_exact: bool,
    exact_async_cost: bool,
}

fn evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
    transitions: &[Transition],
    adjudication: DeferredCompletionAdjudication,
) -> serde_json::Value {
    let presentation = receipt.presentation();
    let shutdown = receipt
        .client_shutdown()
        .expect("deferred completion evidence follows client shutdown validation");
    let cost = receipt.final_frame().cost();
    serde_json::json!({
        "schema": "worth-ui-native-phase-f-deferred-completion-world-v1",
        "presentation_transitions": transitions.iter().map(|transition| format!("{:?}", transition.kind())).collect::<Vec<_>>(),
        "pending_completed_same_request": adjudication.completed_deferred_request,
        "no_recovery_path": adjudication.terminal_without_recovery,
        "source_rgba8": presentation.map(|value| value.source_rgba8()),
        "client_physical_size": presentation.map(|value| value.client_physical_size()),
        "retained_baseline_rgba8": presentation.map(|value| value.retained_baseline_rgba8()),
        "retained_center_rgba8": presentation.map(|value| value.retained_center_rgba8()),
        "retained_payload_exact": adjudication.retained_payload_exact,
        "physical_identity_exact": adjudication.physical_identity_exact,
        "port_crossings": presentation.map(|value| value.port_crossings()),
        "exact_async_cost": adjudication.exact_async_cost,
        "cost": {
            "presented_surfaces": cost.presented_surfaces(),
            "native_resource_cache_misses": cost.native_resource_cache_misses(),
            "cleared_pixels": cost.cleared_pixels(),
            "presented_pixels": cost.presented_pixels(),
            "gpu_writes": cost.gpu_writes(),
            "render_passes": cost.render_passes(),
            "surface_acquisitions": cost.surface_acquisitions(),
            "queue_submissions": cost.queue_submissions(),
            "presents": cost.presents(),
        },
        "asynchronous_handoffs": cost.asynchronous_handoffs(),
        "observation_history_complete": receipt.observation_history_complete(),
        "query_close_complete": shutdown.managed_semantic_resources_complete(),
        "terminal_zero": receipt.terminal_census().is_zero(),
    })
}

fn same_request(left: Transition, right: Transition) -> bool {
    left.attempt() == right.attempt() && left.binding() == right.binding()
}
