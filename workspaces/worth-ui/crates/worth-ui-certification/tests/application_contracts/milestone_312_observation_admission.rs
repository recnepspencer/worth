use worth_ui::facade::measurement_exchange::{
    UiHostMeasurementOutcome, WorthUiHostMeasurementSessionExt,
};
use worth_ui::facade::observation::{
    UiChangeClassificationOutcome, UiHostObservationAdmissionStop, UiHostObservationSuccessorOwner,
    UiObservationAdmissionDenial, UiObservationFamily,
};
use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationReportOutcome,
    WorthUiHostObservationSessionExt,
};
use worth_ui::facade::rebind::UiProducedFactFamily;
use worth_ui::facade::source::{
    WorthUiSourceIngressExt, WorthUiSourceProvider, WorthUiWatcherEvent,
};

use super::host_measurement_fixture::{begin_viewport, measurement_host, viewport_observation};
use super::host_observation_fixture::{batch, pointer, report, source};
use super::mounted_application_lifecycle::published_mounted_world::{
    published_observation_world, published_observation_world_with_host,
};

#[path = "milestone_312_observation_admission/fact_index.rs"]
mod fact_index;
#[path = "milestone_312_observation_admission/source_classification.rs"]
mod source_classification;

#[test]
fn package_bound_source_candidate_enters_one_effect_free_observation() {
    let mut world = published_observation_world("phase-312-source-admission");
    let provider = WorthUiSourceProvider::in_memory("phase-312-source-admission")
        .with_file("app/main.wui", "");
    let mut ingress = world.session.source_event_ingress(provider).start();
    let settled = ingress
        .ingest([WorthUiWatcherEvent::provider_revision(
            "phase-312-source-admission",
        )])
        .expect("one real provider revision settles");
    let candidate = settled
        .attempt_source_rebind(world.session.capabilities())
        .into_candidate_submission()
        .expect("valid source compiles to one sealed candidate");
    let revision = candidate.source_revision().clone();
    let ordering = candidate.ordering_receipt().clone();
    let counters = candidate.counters();
    let composition = candidate.composition_basis().clone();
    let publication = world.session.current_mounted_publication().cloned();
    let host_calls = world.host.presentation_calls();

    let mut turn = world.session.begin_observation_turn().unwrap();
    let receipt = turn.admit_source(candidate).unwrap();
    assert_eq!(receipt.family(), UiObservationFamily::AuthoredSource);
    assert_eq!(receipt.owner_order(), ordering.sequence());
    let admitted = turn.seal().unwrap();
    let source = admitted.observations()[0]
        .source_observation()
        .expect("source family retains a typed source observation");
    assert_eq!(source.revision(), &revision);
    assert_eq!(source.ordering_receipt(), &ordering);
    assert_eq!(source.counters(), counters);
    assert_eq!(source.composition_basis(), &composition);
    assert_eq!(
        admitted.summary().families(),
        &[UiObservationFamily::AuthoredSource]
    );
    assert_eq!(
        world.session.current_mounted_publication(),
        publication.as_ref()
    );
    assert_eq!(world.host.presentation_calls(), host_calls);
    drop(admitted);
    let _ = world.session.shutdown();
}

#[test]
fn validated_host_families_form_one_canonical_effect_free_set() {
    let mut world = published_observation_world("phase-312-host-admission");
    let raw = batch(
        source(&world.session, world.binding, &world.current),
        (1, 2),
        UiHostObservationLoss::Complete,
        vec![
            report(
                1,
                UiHostObservationPayload::Viewport {
                    width_subpixels: 80_000,
                    height_subpixels: 60_000,
                },
                &world.current,
            ),
            report(
                2,
                UiHostObservationPayload::DeviceScale { micros: 1_250_000 },
                &world.current,
            ),
        ],
    );
    let validated = validated(world.session.validate_host_observation_batch(raw));
    let publication = world.session.current_mounted_publication().cloned();
    let host_calls = world.host.presentation_calls();
    let validation_work = world.session.host_observation_work_report();

    let mut turn = world.session.begin_observation_turn().unwrap();
    let receipt = turn.admit_host(validated).unwrap();
    assert!(receipt.unavailable().is_empty());
    assert_eq!(receipt.admitted().len(), 2);
    let admitted = turn.seal().unwrap();
    assert_eq!(
        admitted.summary().families(),
        &[
            UiObservationFamily::HostViewport,
            UiObservationFamily::HostDeviceScale,
        ]
    );
    assert_eq!(
        world.session.current_mounted_publication(),
        publication.as_ref()
    );
    assert_eq!(world.host.presentation_calls(), host_calls);
    assert_eq!(
        world.session.host_observation_work_report(),
        validation_work
    );
    let changed = match world.session.classify_observations(admitted) {
        Ok(UiChangeClassificationOutcome::Changed(changed)) => changed,
        _ => panic!("two supported host families must produce one changed classification"),
    };
    assert_eq!(
        changed
            .facts()
            .iter()
            .map(|fact| fact.family())
            .collect::<Vec<_>>(),
        [
            UiProducedFactFamily::HostViewport,
            UiProducedFactFamily::HostDeviceScale,
        ]
    );
}

#[test]
fn duplicate_family_poisons_the_turn_without_committing_owner_progress() {
    let mut world = published_observation_world("phase-312-host-rollback");
    let first = validated(world.session.validate_host_observation_batch(batch(
        source(&world.session, world.binding, &world.current),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            UiHostObservationPayload::Viewport {
                width_subpixels: 80_000,
                height_subpixels: 60_000,
            },
            &world.current,
        )],
    )));
    let successor = validated(world.session.validate_host_observation_batch(batch(
        source(&world.session, world.binding, &world.current),
        (2, 2),
        UiHostObservationLoss::Complete,
        vec![report(
            2,
            UiHostObservationPayload::Viewport {
                width_subpixels: 81_000,
                height_subpixels: 61_000,
            },
            &world.current,
        )],
    )));

    let mut poisoned = world.session.begin_observation_turn().unwrap();
    poisoned.admit_host(first).unwrap();
    assert!(matches!(
        poisoned.admit_host(successor.clone()),
        Err(UiHostObservationAdmissionStop::Observation(
            UiObservationAdmissionDenial::DuplicateFamily
        ))
    ));
    assert!(matches!(
        poisoned.seal(),
        Err(UiObservationAdmissionDenial::PoisonedTurn)
    ));

    let mut successor_turn = world.session.begin_observation_turn().unwrap();
    successor_turn
        .admit_host(successor)
        .expect("poisoned predecessor committed no owner progress");
    let admitted = successor_turn.seal().unwrap();
    assert_eq!(
        admitted.summary().families(),
        &[UiObservationFamily::HostViewport]
    );
    drop(admitted);
    let _ = world.session.shutdown();
}

#[test]
fn raw_pointer_motion_requires_pointer_presence_owner_progression() {
    let mut world = published_observation_world("phase-312-host-unavailable");
    let raw = batch(
        source(&world.session, world.binding, &world.current),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(1, pointer(1, 10), &world.current)],
    );
    let validated = validated(world.session.validate_host_observation_batch(raw));
    let mut turn = world.session.begin_observation_turn().unwrap();
    let receipt = turn.admit_host(validated.clone()).unwrap();
    assert!(receipt.admitted().is_empty());
    assert_eq!(receipt.unavailable().len(), 1);
    assert_eq!(
        receipt.unavailable()[0].successor(),
        UiHostObservationSuccessorOwner::PointerPresence
    );
    assert!(matches!(
        turn.seal(),
        Err(UiObservationAdmissionDenial::EmptyTurn)
    ));

    let mut turn = world.session.begin_observation_turn().unwrap();
    assert!(matches!(
        turn.admit_required_host(validated),
        Err(UiHostObservationAdmissionStop::RequiredFamilyUnavailable(_))
    ));
    assert!(matches!(
        turn.seal(),
        Err(UiObservationAdmissionDenial::PoisonedTurn)
    ));
}

#[test]
fn solicited_measurement_admission_retains_owner_coordinates_without_effects() {
    let mut world = published_observation_world_with_host(
        "phase-312-measurement-admission",
        measurement_host(),
    );
    let request = begin_viewport(&mut world.session, Some(world.binding), 100, 0);
    let result = match world
        .session
        .complete_host_measurement(viewport_observation(&request, 800.0, 600.0), 1)
    {
        UiHostMeasurementOutcome::Completed(result) => result,
        other => panic!("solicited measurement must complete: {other:?}"),
    };
    let expected_order = request.identity().as_u64();
    let expected_source = world.session.host_session_identity().as_u64();
    let publication = world.session.current_mounted_publication().cloned();
    let host_calls = world.host.presentation_calls();

    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_measurement(result).unwrap();
    let admitted = turn.seal().unwrap();
    let measurement = admitted.observations()[0].measurement().unwrap();
    assert_eq!(measurement.source_identity(), expected_source);
    assert_eq!(measurement.source_order(), expected_order);
    assert_eq!(
        admitted.summary().families(),
        &[UiObservationFamily::Measurement]
    );
    assert_eq!(
        world.session.current_mounted_publication(),
        publication.as_ref()
    );
    assert_eq!(world.host.presentation_calls(), host_calls);
}

fn validated(
    outcome: UiHostObservationReportOutcome,
) -> worth_ui::facade::observation_report::UiValidatedHostObservationBatch {
    match outcome {
        UiHostObservationReportOutcome::Validated(batch) => batch,
        other => panic!("raw host report must validate first: {other:?}"),
    }
}
