use super::{admit, ingest, presence, presentation, publish};
use worth_ui_host_contract::*;

#[test]
fn consequence_owner_validation_accepts_receipt_refresh_but_denies_changed_hover() {
    let role = super::super::fixture::role();
    let (mut session, host) = super::super::fixture::session(&role);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    super::super::close_source_turn(&mut session, &role, "consequence-hover-predecessor");
    publish(&mut session, &host, 1);
    let first = presentation(&session, surface);
    let basis = session.mounted.interaction_hit_test_basis(first).unwrap();
    let row = *basis
        .rows()
        .iter()
        .find(|row| {
            session
                .mounted
                .current_mounted_identity_basis(row.mounted_instance())
                .unwrap()
                .graph_node_identity()
                == graph
        })
        .unwrap();
    let inside = UiHostSurfacePosition::viewport_logical(
        ((row.bounds().platform_box().x() + row.bounds().platform_box().width() / 2.0) * 1_000.0)
            as i64,
        ((row.bounds().platform_box().y() + row.bounds().platform_box().height() / 2.0) * 1_000.0)
            as i64,
    );
    let pointer = UiHostPointerIdentity::new(1);
    admit(&mut session, first, 1, pointer, inside, false);
    let prepared = crate::facade::entry::intent_consequence_observation::prepare_intent_consequence_observation(
        &mut session, crate::runtime::observation::UiIntentConsequenceObservationBatch::new(None, None, None))
        .unwrap_or_else(|_| panic!("close current Hover owner"));
    let (frame, evidence) = session
        .prepare_observed_intent_consequence_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            0,
            Vec::new(),
            prepared.progress,
        )
        .unwrap();
    drop(prepared.set);
    evidence.validate(&session, &frame).unwrap();
    let before = presence(&session);

    // A real accepted frame refreshes receipt identity for the same stationary target.
    publish(&mut session, &host, 2);
    let refreshed = presence(&session);
    assert_ne!(
        before, refreshed,
        "the proof must actually advance owner receipts"
    );
    assert_eq!(
        before.postures()[0].target(),
        refreshed.postures()[0].target()
    );
    evidence.validate(&session, &frame).unwrap();

    // A subsequent real pointer observation changes the consumed Hover class.
    let current = presentation(&session, surface);
    let left = ingest(
        &mut session,
        current,
        2,
        pointer,
        UiHostSurfacePosition::viewport_logical(-1_000, -1_000),
        false,
    );
    assert!(left.pointer_presence_denials().is_empty());
    assert_eq!(left.pointer_presence_transitions().len(), 1);
    assert!(matches!(
        evidence.validate(&session, &frame),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::StaleConsequenceOwnerSnapshot)
    ));
    let calls = host.presentation_calls();
    assert!(matches!(
        session.present_prepared_observed_frame(
            frame,
            &evidence,
            None,
            UiPresentationDeadline::at_tick(u64::MAX),
            3
        ),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::StaleConsequenceOwnerSnapshot)
    ));
    assert_eq!(
        host.presentation_calls(),
        calls,
        "changed Hover denies before host effects"
    );
    let _ = session.shutdown();
}
