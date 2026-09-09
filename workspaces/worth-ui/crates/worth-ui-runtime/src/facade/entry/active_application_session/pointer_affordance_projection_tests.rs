use crate::declaration::UiPointerAffordance as Family;
use crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot;
use worth_ui_host_contract::*;

#[path = "pointer_affordance_reuse_tests.rs"]
mod reuse_tests;

#[test]
fn automatic_pointer_projection_is_role_independent_and_surface_local() {
    let (mut session, _host, surfaces) = mounted_world();
    let main = surfaces[0];
    let baseline = session.intent_admission_metrics();

    motion(
        &mut session,
        main,
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let ready = snapshot(&session);
    assert_eq!(ready.projections().len(), 1);
    assert_row(&ready, main, 1, Family::Activation);
    assert!(ready.projections()[0]
        .operability()
        .unwrap()
        .unwrap()
        .product_decision()
        .unwrap()
        .is_operable());
    assert!(
        session.appearance_owner_snapshot.is_none(),
        "pointer projection has no role dependency"
    );

    session
        .update_intent_boolean_fact(&super::fixture::fact(super::fixture::MUTABLE), false)
        .unwrap();
    close_source(&mut session, "pointer-affordance-readonly");
    let denied = snapshot(&session);
    assert_row(&denied, main, 1, Family::Default);
    assert_eq!(&*denied.changed_surfaces(&ready), &[main]);
    assert_eq!(
        denied.projections()[0]
            .operability()
            .unwrap()
            .unwrap()
            .product_decision()
            .unwrap()
            .primary_cause(),
        Some(crate::runtime::intent::UiIntentInoperableCause::Readonly)
    );

    session
        .update_intent_boolean_fact(&super::fixture::fact(super::fixture::MUTABLE), true)
        .unwrap();
    close_source(&mut session, "pointer-affordance-restored");
    let restored = snapshot(&session);
    assert_row(&restored, main, 1, Family::Activation);
    motion(
        &mut session,
        surfaces[1],
        2,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let neighbors = snapshot(&session);
    assert_eq!(neighbors.projections().len(), 2);
    assert_row(&neighbors, main, 1, Family::Activation);
    assert_row(&neighbors, surfaces[1], 2, Family::Activation);
    assert_eq!(&*neighbors.changed_surfaces(&restored), &[surfaces[1]]);

    motion(
        &mut session,
        surfaces[2],
        3,
        3,
        UiHostPointerDeviceKind::Touch,
        false,
    );
    let touched = snapshot(&session);
    assert_eq!(
        touched.projections().len(),
        2,
        "touch cannot select a cursor on another surface"
    );
    assert!(touched.changed_surfaces(&neighbors).is_empty());

    motion(
        &mut session,
        main,
        4,
        1,
        UiHostPointerDeviceKind::Mouse,
        true,
    );
    let outside = snapshot(&session);
    assert_row(&outside, main, 1, Family::Default);
    assert!(outside
        .projections()
        .iter()
        .find(|row| row.surface() == main)
        .unwrap()
        .target()
        .is_none());
    assert_eq!(&*outside.changed_surfaces(&touched), &[main]);
    motion(
        &mut session,
        main,
        5,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let displaced = snapshot(&session);
    assert_eq!(displaced.projections().len(), 1);
    assert_row(&displaced, main, 2, Family::Activation);
    let mut changed = displaced.changed_surfaces(&outside).into_vec();
    changed.sort_unstable();
    let mut expected = vec![main, surfaces[1]];
    expected.sort_unstable();
    assert_eq!(changed, expected);
    motion(
        &mut session,
        main,
        6,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let unchanged = snapshot(&session);
    assert!(
        unchanged.changed_surfaces(&displaced).is_empty(),
        "sequence-only motion adds no mechanic invalidation"
    );
    assert_eq!(session.intent_admission_metrics(), baseline);
    assert_eq!(session.active_intent_occupancy_count_for_certification(), 0);
    assert!(session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .all(|node| node.appearance_role_attachment().is_none()));
    let _ = session.shutdown();
}

fn snapshot(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> UiPointerAffordanceSnapshot {
    session
        .pointer_affordance_snapshot
        .clone()
        .expect("ordinary observation close seals pointer projection")
}

fn assert_row(
    snapshot: &UiPointerAffordanceSnapshot,
    surface: UiSemanticSurfaceIdentity,
    pointer: u64,
    family: Family,
) {
    let row = snapshot
        .projections()
        .iter()
        .find(|row| row.surface() == surface)
        .unwrap();
    assert_eq!(row.pointer(), UiHostPointerIdentity::new(pointer));
    assert_eq!(row.family(), family);
}

fn close_source(session: &mut crate::facade::WorthUiActiveApplicationSession, name: &str) {
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored(name)
                .with_rust_authored_input(super::fixture::source_with_role(None, 2)),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let set = turn.seal().unwrap();
    session.classify_observations(set).unwrap();
}

fn motion(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    sequence: u64,
    pointer: u64,
    kind: UiHostPointerDeviceKind,
    outside: bool,
) {
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    assert_eq!(hit.rows().len(), 1);
    let bounds = hit.rows()[0].bounds();
    let position = if outside {
        UiHostSurfacePosition::viewport_logical(-1_000, -1_000)
    } else {
        UiHostSurfacePosition::viewport_logical(
            ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
            ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
        )
    };
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("protocol")
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::PointerMotion {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                position,
                pressed_buttons: UiHostPressedPointerButtons::NONE,
            },
        )
        .with_pointer_device_kind(kind)
        .unwrap()],
    })
    .unwrap();
    let outcome = session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = outcome
    else {
        panic!("pointer ingress must reach owner: {outcome:?}")
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    if receipt.pointer_presence_transitions().is_empty() {
        close_source(
            session,
            &format!("pointer-affordance-motion-{}", sequence.value()),
        );
    } else {
        let mut turn = session.begin_observation_turn().unwrap();
        for transition in receipt.pointer_presence_transitions() {
            turn.admit_pointer_presence_transition(transition.clone())
                .unwrap();
        }
        let set = turn.seal().unwrap();
        session.classify_observations(set).unwrap();
    }
}

fn mounted_world() -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
    Vec<UiSemanticSurfaceIdentity>,
) {
    let role = super::fixture::role();
    let source = super::fixture::source_with_role(None, 2);
    let (mut session, host) = super::fixture::session_with_source(&role, source);
    let graph = session
        .graph()
        .node_identities()
        .find(|identity| {
            session
                .graph()
                .lookup()
                .graph_node(*identity)
                .is_some_and(|node| {
                    node.value().declaration_identity().authored_semantic_name()
                        != "worth_ui.runtime.bootstrap.product_root"
                })
        })
        .unwrap();
    let (main, _) = super::super::mounting_fixture::mount_graph_node(&mut session, 1_000, graph);
    let mut surfaces = vec![main];
    for _ in 0..2 {
        let surface = session.create_semantic_surface().unwrap();
        session
            .register_host_surface(
                surface,
                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                crate::facade::mounted::UiSurfaceBindingProfile::new(
                    1_000,
                    crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let node = session.mounted_graph_node(graph).unwrap();
        session.mount_instance(node, surface).unwrap();
        crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
            &mut session,
            surface,
        );
        surfaces.push(surface);
    }
    session.advance_mounted_identity_frame().unwrap();
    let frame = super::prepare(&mut session);
    super::publish(&mut session, &host, frame, 1);
    (session, host, surfaces)
}

#[path = "pointer_affordance_mounted_tests.rs"]
mod mounted_tests;
