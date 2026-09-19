use worth_ui::facade::{
    intent::{
        UiIntentConsequencePublicationOutcome, UiIntentDefinition,
        UiIntentExecutionDispatchOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
};

use super::super::only_transition;
use crate::intent::{admission::phase3::world::AdmissionWorld, operability::PrimaryIntent};

pub(in crate::intent::execution::portal_service) fn motion_tick_batch(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    presentation: worth_ui::facade::observation_report::UiHostObservationPresentationBasis,
    sequence: u64,
    tick: u64,
) -> worth_ui::facade::observation_report::UiHostObservationBatch {
    use worth_ui::facade::observation_report::{
        UiHostObservationBatchInput, UiHostObservationLoss, UiHostObservationPayload,
        UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
        UiHostObservationTimeBasis, UiHostProtocolContract, UiHostProtocolNegotiation,
    };
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol must negotiate"),
    };
    let report = UiHostObservationReport::new(
        UiHostObservationSequence::new(sequence),
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
        UiHostObservationPayload::Tick { tick },
    );
    worth_ui::facade::observation_report::UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: session.host_session_identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(
            UiHostObservationSequence::new(sequence),
            UiHostObservationSequence::new(sequence),
        ),
        loss: UiHostObservationLoss::Complete,
        reports: vec![report],
    })
    .expect("one Tick report is a valid raw observation batch")
}

pub(in crate::intent::execution::portal_service) fn assert_motion_tick_applied(
    outcome: worth_ui::facade::interaction::UiHostInteractionIngressOutcome,
) {
    match outcome {
        worth_ui::facade::interaction::UiHostInteractionIngressOutcome::Applied(_) => {}
        other => panic!("validated motion Tick was not applied: {other:?}"),
    }
}

pub(in crate::intent::execution::portal_service) fn scripted_motion_host(
) -> worth_ui_runtime::certification_support::ScriptedPresentationHost {
    use worth_ui_host_contract::{WorthUiHostCapability, WorthUiHostCapabilityReport};

    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(WorthUiHostCapabilityReport::available(vec![
        WorthUiHostCapability::MountedFrameRecording,
        WorthUiHostCapability::ViewportObservation,
    ]));
    host
}

pub(in crate::intent::execution::portal_service) fn launch_scripted_motion_world(
    host: worth_ui_runtime::certification_support::ScriptedPresentationHost,
) -> AdmissionWorld {
    use crate::intent::operability::build_open_portal_application_with_host;

    let (application, facts) = build_open_portal_application_with_host(host);
    let mut world = AdmissionWorld::launch_application_on_declared_surface(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
    );
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, super::super::super::execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(&mut world)
        .into_consequence()
        .expect("completed portal intent retains its mounted consequence");
    assert!(matches!(
        world.session.publish_intent_consequences(
            handle,
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(40),
        ),
        UiIntentConsequencePublicationOutcome::Published(_)
    ));
    world
}
