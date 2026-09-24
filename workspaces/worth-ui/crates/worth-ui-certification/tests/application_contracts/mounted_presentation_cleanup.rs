use worth_ui_runtime::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountWorkClass,
    UiMountedFrameOutcome, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared,
};
use super::mounted_host_protocol::scripted_host::{
    ScriptedPresentationHost, ScriptedSurfaceCompletion as UiHostSurfaceInFlightCompletion,
};

#[test]
fn start_time_indeterminacy_cancels_every_earlier_in_flight_surface() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "start-indeterminate-cleanup", 2);
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
    let frame = prepared(&mut session);

    let outcome =
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);

    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_eq!(
        host.cancellation_calls().len(),
        1,
        "terminal start-time indeterminacy must not abandon an earlier host token"
    );
    assert!(session.inspect_mounted_identity().current_frame().is_none());
}

#[test]
fn completion_time_indeterminacy_cancels_every_other_pending_surface() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "completion-indeterminate-cleanup", 2);
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::PresentationIndeterminate],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let frame = prepared(&mut session);
    let started =
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);
    let UiMountedFrameOutcome::InFlight(in_flight) = started else {
        panic!("both host surfaces accepted asynchronous work");
    };

    let outcome = session.complete_mounted_presentation(in_flight, 1);

    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_eq!(
        host.cancellation_calls().len(),
        1,
        "terminal completion-time indeterminacy must drain every sibling token"
    );
    assert!(session.inspect_mounted_identity().current_frame().is_none());
}

#[test]
fn indeterminate_cost_preserves_every_adapter_translation_already_performed() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "indeterminate-adapter-cost", 2);
    host.push_presentation(completion_with_cost(3, 30, true));
    host.push_presentation(ScriptedPresentationOutcome::Presented(
        ScriptedPresentationAcknowledgement::new(
            UiHostSurfacePresentationMode::RecordOnly,
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(2),
            worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                worth_ui_host_contract::UiMountedEffectFamily::RecordedProjection,
                worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
            ]),
            adapter_cost(7, 70, false),
        ),
    ));

    let frame = prepared(&mut session);
    let outcome =
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);
    let cost = outcome
        .cost_report()
        .expect("indeterminate frame owns terminal cost evidence");
    assert_eq!(
        cost.work_class(),
        UiMountWorkClass::IndeterminatePresentation
    );
    assert_eq!(cost.adapter().presented_surfaces(), 2);
    assert_eq!(cost.adapter().translated_rows(), 10);
    assert_eq!(cost.adapter().translated_bytes(), 100);
    assert_eq!(cost.adapter().native_resource_cache_hits(), 1);
    assert_eq!(cost.adapter().native_resource_cache_misses(), 1);
    assert_eq!(cost.named().presented(), 2);
    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
}

fn completion_with_cost(rows: u64, bytes: u64, cache_hit: bool) -> ScriptedPresentationOutcome {
    ScriptedPresentationOutcome::Presented(ScriptedPresentationAcknowledgement::new(
        UiHostSurfacePresentationMode::RecordOnly,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
        crate::mounted_host_protocol::scripted_host::recorded_effects(),
        adapter_cost(rows, bytes, cache_hit),
    ))
}

fn adapter_cost(
    rows: u64,
    bytes: u64,
    cache_hit: bool,
) -> worth_ui_host_contract::UiHostPresentationCostReport {
    worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
        worth_ui_host_contract::UiHostPresentationCostInput {
            presented_surfaces: 1,
            translated_rows: rows,
            translated_bytes: bytes,
            native_resource_cache_hits: u64::from(cache_hit),
            native_resource_cache_misses: u64::from(!cache_hit),
            asynchronous_handoffs: 0,
            ..Default::default()
        },
    )
}
