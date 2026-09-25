use super::*;
use crate::runtime::interaction::UiHostInteractionIngressOutcome;
#[test]
fn accepted_sample_completion_does_not_overwrite_staged_direct_geometry() {
    use crate::runtime::scroll::UiScrollDeltaCause;

    let mut declared = smooth_scroll(true);
    declared.region = declared
        .region
        .clone()
        .with_scroll_chrome(super::super::scroll_chrome_fixture::contract());
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_with_scroll(declared));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    scroll.world.host.push_in_flight(
        vec![super::pending_sample::presented_sample()],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    super::super::scroll_settle_frame::settle_scripted_frame(&mut scroll, 7);
    let Some(crate::mounting::UiMountedMotionSampleSettlement::Committed(sampling)) = scroll
        .world
        .session
        .mounted
        .complete_motion_sample_presentation(&scroll.world.session.host_session)
    else {
        panic!("the in-flight sample commits")
    };
    let presented = sampling
        .presented_surface()
        .expect("a committed sample names the surface its witness proved");
    // The witnessed sample is accepted while direct input stages a successor.
    let staged = UiScrollOffset::new(0, 10_000).unwrap();
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            scroll.owner,
            scroll.incarnation,
            scroll.target(),
            0,
            staged,
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .unwrap();
    // The settle cannot land over staged direct geometry: it is owed, not lost.
    assert_eq!(
        scroll
            .world
            .session
            .settle_accepted_scroll_sample(presented),
        UiScrollSettleDisposition::DeferredPendingGeometry
    );
    assert!(scroll.world.session.awaits_scroll_settle_retry());
    // An unrelated input cannot replay that old sample.
    let presentation = scroll.presentation();
    scroll.world.host.enqueue_observation_for_next_drain(
        super::super::pointer_geometry::pointer_batch(
            scroll.world.session.host_session.identity().as_u64(),
            presentation,
            1,
            [150_000, 55_000],
        ),
    );
    let outcomes = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    assert!(matches!(
        outcomes.as_ref(),
        [UiHostInteractionIngressOutcome::Applied(_)]
    ));
    assert!(scroll.world.session.awaits_scroll_settle_retry());
    scroll.publish_direct(9);
    assert_eq!(scroll.accepted_offset(), staged);
    let nested = scroll.world.instances[2];
    let presented = scroll
        .world
        .host
        .last_node_changes()
        .into_iter()
        .find_map(|change| match change {
            UiMountedPresentationNodeChange::Upsert(state)
                if state.mounted_instance() == nested =>
            {
                Some(state)
            }
            _ => None,
        })
        .expect("direct publication must present the nested scrolled occurrence");
    let UiMountedAllocationProjection::Known { bounds, .. } = presented.allocation() else {
        panic!("nested occurrence has an allocated host rectangle")
    };
    assert_eq!(bounds.y(), 52.0);
    if let UiMountedPresentationNodePaint::Command(command) = presented.paint() {
        assert!(
            scroll
                .world
                .host
                .last_appearance_samples()
                .iter()
                .all(|sample| {
                    sample.command() != command
                        || sample
                            .transform()
                            .is_none_or(|transform| transform.source() == transform.sampled())
                }),
            "a stale sample must not displace the host's direct geometry"
        );
    }
    assert_eq!(scroll.mounted_offset(), Some(staged));
    // Once the direct geometry is presented, the owed settle is paid without
    // displacing it.
    assert_eq!(
        scroll.world.session.settle_owed_scroll_samples(),
        UiScrollSettleDisposition::Idle,
        "the landed direct page retired the owed sample on its own surface"
    );
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.mounted_offset(), Some(staged));
    let _ = scroll.world.session.shutdown();
}
