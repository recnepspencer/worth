use crate::facade::intent::*;
use worth_ui_host_contract::*;

#[path = "pointer_confirmation_fixture.rs"]
mod fixture;

#[test]
fn stationary_confirmation_expiry_reaches_mounted_pointer_without_activation_or_consumption() {
    confirmation_expiry(ExpiryObservation::Manual);
}

#[test]
fn native_clock_overrides_caller_time_and_shell_idle_close_preserves_expiry() {
    confirmation_expiry(ExpiryObservation::Native);
}

#[test]
fn native_idle_expiry_automatically_commits_pointer_output() {
    confirmation_expiry(ExpiryObservation::Driver(
        crate::native_platform::PointerExpiryPresentation::Immediate,
    ));
}

#[test]
fn native_pointer_refresh_retains_in_flight_work_without_advancing_the_program() {
    confirmation_expiry(ExpiryObservation::Driver(
        crate::native_platform::PointerExpiryPresentation::PendingRefresh,
    ));
}

#[test]
fn native_pointer_refresh_retains_rejected_work_until_readiness() {
    confirmation_expiry(ExpiryObservation::Driver(
        crate::native_platform::PointerExpiryPresentation::RejectedRefresh,
    ));
}

#[test]
fn native_expiry_survives_an_older_program_presentation() {
    confirmation_expiry(ExpiryObservation::Driver(
        crate::native_platform::PointerExpiryPresentation::PendingProgram,
    ));
}

#[test]
fn native_expiry_uses_the_custom_application_owner_and_preserves_its_close_directive() {
    confirmation_expiry(ExpiryObservation::Custom);
}

enum ExpiryObservation {
    Manual,
    Custom,
    Native,
    Driver(crate::native_platform::PointerExpiryPresentation),
}

fn confirmation_expiry(observation: ExpiryObservation) {
    let (mut shell, host) = fixture::shell();
    let mut session = shell.session.as_mut();
    let surface =
        session.inspect_mounted_identity().surface_bindings()[0].semantic_surface_identity();
    session.advance_mounted_identity_frame().unwrap();
    let frame = super::prepare(&mut session);
    super::publish(&mut session, &host, frame, 1);
    let product = target_position(&session, surface, TargetRoute::Product);
    let mut activation = None;
    for (sequence, transition, pressed) in [
        (1, UiHostPointerButtonTransition::Pressed, true),
        (2, UiHostPointerButtonTransition::Released, false),
    ] {
        let receipt = ingress(
            &mut session,
            surface,
            sequence,
            product,
            Some(transition),
            pressed,
        );
        for transition in receipt.into_transitions().into_vec() {
            if let crate::facade::interaction::UiInteractionTransition::Semantic(interaction) =
                transition
            {
                activation = Some(interaction);
            }
        }
    }
    let UiIntentRouteResolution::Product(route) = session
        .resolve_intent_route(
            crate::facade::interaction::UiIntentRouteSource::mounted_interaction(
                activation.unwrap(),
            ),
        )
        .unwrap()
    else {
        panic!("product route");
    };
    let payload = session.prepare_intent_payload(route).unwrap();
    let UiIntentOperabilityOutcome::Inoperable(candidate) =
        session.evaluate_intent_operability(payload)
    else {
        panic!("requires confirmation");
    };
    let UiIntentConfirmationIssueOutcome::Pending(pending) =
        session.issue_intent_confirmation(candidate)
    else {
        panic!("one real pending challenge");
    };
    let frame = super::prepare(&mut session);
    super::publish(&mut session, &host, frame, 2);
    let confirmation = target_position(&session, surface, TargetRoute::Confirmation);
    let receipt = ingress(&mut session, surface, 3, confirmation, None, false);
    assert!(
        receipt.transitions().is_empty(),
        "hover must produce no activation"
    );
    assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    let baseline = session.intent_confirmation_metrics();
    let admission = session.intent_admission_metrics();
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_pointer_presence_transition(receipt.pointer_presence_transitions()[0].clone())
        .unwrap();
    let set = turn.seal_at_host_time(monotonic(3)).unwrap();
    session.classify_observations(set).unwrap();
    assert_eq!(
        session
            .pointer_affordance_snapshot
            .as_ref()
            .unwrap()
            .confirmation_deadline(),
        pending.expires_at_millis().checked_add(1)
    );
    let frame = super::prepare(&mut session);
    assert_pointer(&frame, UiPointerAffordanceFamily::Activation);
    super::publish(&mut session, &host, frame, 3);
    assert!(matches!(
        session.begin_observation_turn().unwrap().seal(),
        Err(crate::facade::observation::UiObservationAdmissionDenial::EmptyTurn)
    ));

    if matches!(observation, ExpiryObservation::Custom) {
        crate::native_platform::exercise_custom_pointer_expiry(
            shell,
            host,
            pending.expires_at_millis(),
            |shell| {
                let pointer = shell
                    .session
                    .mounted
                    .current_pointer_affordance_for_test(surface)
                    .unwrap();
                assert_eq!(pointer.family(), UiPointerAffordanceFamily::Default);
                assert_eq!(shell.session.intent_confirmation_metrics(), baseline);
                assert_eq!(shell.session.intent_admission_metrics(), admission);
            },
        );
        return;
    }

    if let ExpiryObservation::Driver(presentation) = observation {
        let shell = crate::native_platform::exercise_pointer_expiry(
            shell,
            host,
            pending.expires_at_millis(),
            super::super::support::LEGACY_STATIC_PAINT_TOKEN,
            presentation,
        );
        let pointer = shell
            .session
            .mounted
            .current_pointer_affordance_for_test(surface)
            .unwrap();
        assert_eq!(
            pointer.family(),
            UiPointerAffordanceFamily::Default,
            "automatic handoff committed the expired successor"
        );
        assert_eq!(pointer.surface(), surface);
        assert_eq!(pointer.pointer(), UiHostPointerIdentity::new(1));
        assert_eq!(shell.session.intent_confirmation_metrics(), baseline);
        assert_eq!(shell.session.intent_admission_metrics(), admission);
        let _ = shell.shutdown();
        return;
    }

    if matches!(observation, ExpiryObservation::Native) {
        // Simulate an elapsed native epoch without waiting out the challenge TTL.
        // Input/host presentation are scripted; sampling and close are production.
        let clock = worth_ui_host_native::UiNativeObservationClock::from_certification_elapsed(
            pending.expires_at_millis() + 1,
        )
        .unwrap();
        shell
            .install_native_observation_clock(clock.clone())
            .unwrap();
        assert!(shell.install_native_observation_clock(clock).is_err());
        let session = shell.session.as_mut();
        timed_close(session, 3);
        let frame = super::prepare(session);
        assert_pointer(&frame, UiPointerAffordanceFamily::Default);
        super::publish(session, &host, frame, 4);
        let presentations = host.presentation_calls();
        assert_eq!(shell.close_native_observation_time(), Ok(None));
        let session = shell.session.as_mut();
        let frame = super::prepare(session);
        frame.assert_no_unpublished_appearance_for_test();
        drop(frame);
        assert_eq!(host.presentation_calls(), presentations);
        assert_eq!(session.intent_confirmation_metrics(), baseline);
        assert_eq!(session.intent_admission_metrics(), admission);
        let _ = shell.shutdown();
        return;
    }

    timed_close(&mut session, pending.expires_at_millis());
    let frame = super::prepare(&mut session);
    frame.assert_no_unpublished_appearance_for_test();
    drop(frame);
    timed_close(&mut session, pending.expires_at_millis() + 1);
    assert_eq!(
        session
            .pointer_affordance_snapshot
            .as_ref()
            .unwrap()
            .confirmation_deadline(),
        None
    );
    let frame = super::prepare(&mut session);
    assert_pointer(&frame, UiPointerAffordanceFamily::Default);
    super::publish(&mut session, &host, frame, 4);
    timed_close(&mut session, pending.expires_at_millis() + 2);
    let frame = super::prepare(&mut session);
    frame.assert_no_unpublished_appearance_for_test();
    drop(frame);
    assert_eq!(session.intent_confirmation_metrics(), baseline);
    assert_eq!(session.intent_admission_metrics(), admission);
    let _ = shell.shutdown();
}

fn timed_close(session: &mut crate::facade::WorthUiActiveApplicationSession, millis: u64) {
    let set = session
        .begin_observation_turn()
        .unwrap()
        .seal_at_host_time(monotonic(millis))
        .unwrap();
    assert!(
        set.observations().is_empty(),
        "expiry close has no synthetic input"
    );
    session.classify_observations(set).unwrap();
}

fn monotonic(millis: u64) -> UiHostObservationTimeBasis {
    UiHostObservationTimeBasis::HostMonotonicMillis(millis)
}

fn assert_pointer(
    frame: &crate::mounting::UiPreparedMountedFrame,
    family: UiPointerAffordanceFamily,
) {
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), 1);
    let [UiMountedAppearanceMechanic::Pointer(pointer)] =
        output.fragments()[0].work().successor().mechanics()
    else {
        panic!("one pointer mechanic");
    };
    assert_eq!(pointer.family(), family);
}

enum TargetRoute {
    Product,
    Confirmation,
}

fn target_position(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    expected: TargetRoute,
) -> UiHostSurfacePosition {
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    let row = hit
        .rows()
        .iter()
        .find(|row| {
            let basis = session
                .mounted
                .current_mounted_identity_basis(row.mounted_instance())
                .unwrap();
            let route = session
                .application
                .prepared_authority()
                .intent_catalog()
                .lookup(
                    basis.graph_node_identity(),
                    crate::capability::UiSemanticInteractionFamily::Activate,
                );
            matches!(
                (&expected, route),
                (
                    TargetRoute::Product,
                    Some((
                        crate::declaration::UiIntentCatalogResolvedRoute::Product { .. },
                        _
                    ))
                ) | (
                    TargetRoute::Confirmation,
                    Some((
                        crate::declaration::UiIntentCatalogResolvedRoute::Confirmation { .. },
                        _
                    ))
                )
            )
        })
        .unwrap();
    let bounds = row.bounds();
    UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1000.0) as i64,
    )
}

fn ingress(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    sequence: u64,
    position: UiHostSurfacePosition,
    button: Option<UiHostPointerButtonTransition>,
    pressed: bool,
) -> crate::facade::interaction::UiInteractionBatchReceipt {
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let batch = super::super::pointer_tests::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        sequence,
        UiHostPointerIdentity::new(1),
        position,
        button,
        pressed,
    );
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
        session.admit_host_interaction_batch(batch)
    else {
        panic!("real input admission");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    receipt
}
