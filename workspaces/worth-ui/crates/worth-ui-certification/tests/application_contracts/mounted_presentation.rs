use worth_ui_runtime::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared, InFlightPresentationWorld,
};
use super::mounted_application_lifecycle::known_empty_surface_world::mounted_application_with_host;
use super::mounted_host_protocol::scripted_host::{
    ScriptedPresentationHost, ScriptedSurfaceCompletion as UiHostSurfaceInFlightCompletion,
};
use super::mounted_presentation_model_trace::assert_model_outcome;
use super::mounted_protocol_model::{ModelCancellation, ModelPresentation, ModelSurfaceStart};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountedFrameOutcome,
    UiPresentationDeadline,
};

#[path = "mounted_presentation/query_uncertainty.rs"]
mod query_uncertainty;
#[path = "mounted_presentation/support.rs"]
mod support;
use support::{
    present_asynchronously, present_synchronously, publish_partial_effects,
    rebind_and_reconcile_affected,
};

#[test]
fn synchronous_and_multi_in_flight_completion_converge_with_distinct_attempts() {
    let host = ScriptedPresentationHost::default();
    let (mut session, bindings) = mounted_session(host.clone(), "presentation-convergence", 2);
    let synchronous = present_synchronously(&mut session, &host);
    let asynchronous = present_asynchronously(&mut session, &host);

    assert_ne!(synchronous.attempt(), asynchronous.attempt());
    assert_ne!(synchronous.frame(), asynchronous.frame());
    assert_eq!(synchronous.bindings(), asynchronous.bindings());
    assert_eq!(synchronous.bindings(), bindings);
}

#[test]
fn partial_effects_block_the_semantic_surface_until_reset_binding_evidence() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "presentation-reconcile", 2);
    let affected = publish_partial_effects(&mut session, &host);
    rebind_and_reconcile_affected(&mut session, &affected);

    let retry = prepared(&mut session);
    host.push_presented();
    host.push_presented();
    assert!(matches!(
        session.present_prepared_mounted_frame(retry, UiPresentationDeadline::at_tick(10), 0,),
        UiMountedFrameOutcome::Published(_)
    ));
}

#[test]
fn deterministic_deadlines_distinguish_zero_effect_cancellation_from_partial_effects() {
    let host = ScriptedPresentationHost::default();
    let (mut one_surface, _) = mounted_session(host.clone(), "presentation-cancel-empty", 1);
    let frame = prepared(&mut one_surface);
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let mut one_surface_model = ModelPresentation::start(&[ModelSurfaceStart::InFlight]);
    let first_outcome =
        one_surface.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(2), 0);
    assert_model_outcome(&one_surface_model, &first_outcome);
    let in_flight = expect_in_flight(first_outcome);
    one_surface_model.cancel(0, ModelCancellation::CancelledBeforeEffects);
    let cancelled = one_surface.complete_mounted_presentation(in_flight, 2);
    assert_model_outcome(&one_surface_model, &cancelled);

    let second_host = ScriptedPresentationHost::default();
    let (mut two_surfaces, _) =
        mounted_session(second_host.clone(), "presentation-cancel-partial", 2);
    let frame = prepared(&mut two_surfaces);
    second_host.push_presented();
    second_host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let mut two_surface_model =
        ModelPresentation::start(&[ModelSurfaceStart::Presented, ModelSurfaceStart::InFlight]);
    let first_outcome =
        two_surfaces.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(2), 0);
    assert_model_outcome(&two_surface_model, &first_outcome);
    let in_flight = expect_in_flight(first_outcome);
    two_surface_model.cancel(1, ModelCancellation::CancelledBeforeEffects);
    let cancelled = two_surfaces.complete_mounted_presentation(in_flight, 2);
    assert_model_outcome(&two_surface_model, &cancelled);
}

#[test]
fn explicit_cancellation_drains_every_pending_host_token() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "presentation-cancel-all", 2);
    let frame = prepared(&mut session);
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    host.push_in_flight(
        vec![UiHostSurfaceInFlightCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );

    let in_flight = expect_in_flight(session.present_prepared_mounted_frame(
        frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));
    let cancelled = session.cancel_mounted_presentation(in_flight);

    assert!(matches!(
        cancelled,
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_eq!(
        host.cancellation_calls().len(),
        2,
        "one indeterminate token must not prevent later tokens from being cancelled"
    );
}

#[test]
fn shutdown_classifies_discarded_in_flight_observation_before_host_release() {
    let world = InFlightPresentationWorld::accepted("presentation-shutdown");
    drop(world.handle);

    let shutdown = world.session.shutdown();
    let attempts = shutdown.mounted_presentation().attempts();
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        attempts[0].disposition(),
        worth_ui_runtime::facade::mounted::UiMountedPresentationShutdownDisposition::PresentationIndeterminate
    );
    assert_eq!(attempts[0].affected_bindings().len(), 1);
}

#[test]
fn protocol_and_capability_drift_deny_before_any_surface_effect() {
    let foreign_host = ScriptedPresentationHost::default();
    foreign_host.set_protocol(protocol(
        worth_ui_host_contract::UiHostProtocolIdentity::from_untrusted(7),
        2,
    ));
    let launch = mounted_application_with_host("foreign-protocol", foreign_host.clone()).launch();
    assert!(matches!(
        launch,
        Err(
            worth_ui_runtime::facade::runtime_handoff::WorthUiRuntimeLaunchDenial::HostProtocol(
                worth_ui_host_contract::UiHostProtocolDenial::ForeignIdentity
            )
        )
    ));
    assert_eq!(foreign_host.presentation_calls(), 0);

    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "late-host-drift", 2);
    let protocol_frame = prepared(&mut session);
    host.set_protocol(protocol(
        worth_ui_host_contract::UiHostProtocolIdentity::worth_ui(),
        1,
    ));
    assert_rejected_with(
        session.present_prepared_mounted_frame(
            protocol_frame,
            UiPresentationDeadline::at_tick(10),
            0,
        ),
        worth_ui_host_contract::UiHostSurfacePresentationDenial::Protocol(
            worth_ui_host_contract::UiHostProtocolDenial::ProtocolTooOld,
        ),
        2,
    );
    assert_eq!(host.presentation_calls(), 0);

    host.set_protocol(worth_ui_host_contract::UiHostProtocolContract::current());
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(Vec::new()),
    );
    let capability_frame = prepared(&mut session);
    assert_rejected_with(
        session.present_prepared_mounted_frame(
            capability_frame,
            UiPresentationDeadline::at_tick(10),
            0,
        ),
        worth_ui_host_contract::UiHostSurfacePresentationDenial::CapabilityProfileChanged,
        2,
    );
    assert_eq!(host.presentation_calls(), 0);
}

#[test]
fn adapter_overreported_effects_cannot_publish_as_exact_completion() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "presentation-extra-effect", 1);
    let frame = prepared(&mut session);
    host.push_presentation(ScriptedPresentationOutcome::Presented(
        ScriptedPresentationAcknowledgement::new(
            UiHostSurfacePresentationMode::RecordOnly,
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
            worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                worth_ui_host_contract::UiMountedEffectFamily::RecordedProjection,
                worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
            ]),
            worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
                worth_ui_host_contract::UiHostPresentationCostInput {
                    presented_surfaces: 1,
                    translated_rows: 0,
                    translated_bytes: 0,
                    native_resource_cache_hits: 0,
                    native_resource_cache_misses: 0,
                    asynchronous_handoffs: 0,
                    ..Default::default()
                },
            ),
        ),
    ));

    let model = ModelPresentation::start(&[ModelSurfaceStart::EffectStateUnknown]);
    let outcome =
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);
    assert_model_outcome(&model, &outcome);
    assert!(session.inspect_mounted_identity().current_frame().is_none());
}

fn published(
    outcome: UiMountedFrameOutcome,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(value) => value,
        _ => panic!("scripted complete presentation must converge"),
    }
}

fn expect_in_flight(
    outcome: UiMountedFrameOutcome,
) -> worth_ui_runtime::facade::mounted::UiMountedPresentationInFlight {
    match outcome {
        UiMountedFrameOutcome::InFlight(value) => value,
        _ => panic!("scripted pending presentation remains in flight"),
    }
}

fn assert_rejected_with(
    outcome: UiMountedFrameOutcome,
    expected: worth_ui_host_contract::UiHostSurfacePresentationDenial,
    surface_count: usize,
) {
    let cost = outcome
        .cost_report()
        .expect("rejected frame owns terminal cost evidence");
    assert_eq!(
        cost.work_class(),
        worth_ui_runtime::facade::mounted::UiMountWorkClass::RejectedPresentation
    );
    assert_eq!(cost.named().rejected(), surface_count as u64);
    let UiMountedFrameOutcome::RejectedBeforeEffects(rejected) = outcome else {
        panic!("late host drift must reject before effects");
    };
    assert_eq!(rejected.rejections().len(), surface_count);
    let observed = rejected
        .rejections()
        .iter()
        .map(|rejection| rejection.denial())
        .collect::<Vec<_>>();
    assert!(
        observed.iter().all(|denial| *denial == expected),
        "expected every denial to be {expected:?}, observed {observed:?}"
    );
}

fn protocol(
    identity: worth_ui_host_contract::UiHostProtocolIdentity,
    revision: u16,
) -> worth_ui_host_contract::UiHostProtocolContract {
    worth_ui_host_contract::UiHostProtocolContract::new(
        identity,
        worth_ui_host_contract::UiHostProtocolVersion::new(revision),
        worth_ui_host_contract::UiMountedFrameSchemaVersion::new(revision),
        worth_ui_host_contract::UiMountedPresentationSchemaVersion::new(revision),
        worth_ui_host_contract::UiHostProtocolContract::current().observation(),
        worth_ui_host_contract::UiHostMeasurementSchemaVersion::new(revision),
        worth_ui_host_contract::UiHostProtocolContract::current().solicited_effect(),
    )
}
