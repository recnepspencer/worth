use super::*;

#[test]
fn admitted_pointer_batches_carry_hit_miss_and_gesture_query_work() {
    let role = super::super::fixture::role();
    let (mut session, host) = super::super::fixture::session(&role);
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    super::super::close_source_turn(&mut session, &role, "pointer-query-work");
    publish(&mut session, &host, 1);
    let basis = presentation(&session, surface);
    let pointer = UiHostPointerIdentity::new(1);
    // The mounting fixture supplies first-occurrence geometry [8, 12, 28, 20].
    // Its negative-coordinate miss and interior hit are independent of targeting.
    let inside = UiHostSurfacePosition::viewport_logical(16_000, 20_000);
    let outside = UiHostSurfacePosition::viewport_logical(-1_000, -1_000);
    let missed = ingest(&mut session, basis, 1, pointer, outside, false);
    assert!(missed.pointer_presence_denials().is_empty(), "{missed:?}");
    assert_eq!(missed.pointer_presence_transitions().len(), 1, "{missed:?}");
    assert_eq!(missed.pointer_presence_transitions()[0].current(), None);
    assert_query_work(missed.targeting_work());
    let hit = ingest(&mut session, basis, 2, pointer, inside, false);
    assert!(hit.pointer_presence_denials().is_empty(), "{hit:?}");
    assert_eq!(hit.pointer_presence_transitions().len(), 1, "{hit:?}");
    assert!(hit.pointer_presence_transitions()[0].current().is_some());
    assert_query_work(hit.targeting_work());
    let pressed = ingest(&mut session, basis, 3, pointer, inside, true);
    assert_eq!(session.interaction_state().active_gestures(), 1);
    assert_query_work(pressed.targeting_work());
    let release = super::super::pointer_batch(
        session.host_session.identity().as_u64(),
        basis,
        4,
        pointer,
        inside,
        Some(UiHostPointerButtonTransition::Released),
        true,
    );
    let crate::runtime::interaction::UiHostInteractionIngressOutcome::Applied(released) =
        session.admit_host_interaction_batch(release)
    else {
        panic!("release reaches production admission");
    };
    assert_eq!(session.interaction_state().active_gestures(), 0);
    assert_query_work(released.targeting_work());
    let invalid = UiHostSurfacePosition::new(
        UiHostSurfacePositionBasis::new(
            UiHostSurfaceCoordinateSpace::Viewport,
            UiHostSurfaceCoordinateUnit::PhysicalPixel,
        ),
        50_000,
        50_000,
    );
    let denied = ingest(&mut session, basis, 5, pointer, invalid, false);
    assert!(!denied.pointer_presence_denials().is_empty());
    assert_eq!(
        denied.targeting_work(),
        Default::default(),
        "position admission denies before querying"
    );
    let _ = session.shutdown();
    assert_eq!(host.pending_presentation_count(), 0);
}

fn assert_query_work(work: crate::mounting::UiHitTestSpatialWork) {
    assert!(
        work.node_visits() > 0,
        "a hit or miss must visit the admitted spatial index: {work:?}"
    );
    assert_eq!(work.reconstructed_rows(), 0);
    assert_eq!(work.node_copies(), 0);
    assert_eq!(work.map_node_copies(), 0);
}
