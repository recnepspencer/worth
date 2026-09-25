//! A tick is sampled from the tracks as they stood when it was prepared, and
//! lands only when its presentation completes. Whatever the owner did in
//! between must survive that landing.

use super::*;

fn terminal_tick_in_flight(
    world: &World,
    sampler: &mut UiMountedMotionSampler,
) -> (
    UiPreparedMotionSampling,
    crate::runtime::motion::UiMotionTrackIdentity,
) {
    sampler.install(world.receipt(1, 0.0, None)).unwrap();
    commit_tick(sampler, 1, world.presentation);
    let prepared = sampler.prepare_tick(10_000, world.presentation).unwrap();
    let terminals = prepared.receipt().terminals();
    assert_eq!(terminals.len(), 1);
    let track = terminals[0].track();
    (prepared, track)
}

#[test]
fn a_track_installed_while_a_tick_is_in_flight_survives_its_commit() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let (prepared, displaced) = terminal_tick_in_flight(&world, &mut sampler);

    sampler.install(world.receipt(2, 50.0, None)).unwrap();
    let receipt = sampler.commit_prepared(prepared.presented_for_certification());

    assert!(
        receipt.terminals().is_empty(),
        "the displaced track's terminal was already settled by its successor"
    );
    assert!(receipt.samples().is_empty());
    assert!(!sampler.contains_track(displaced));
    assert!(sampler.has_active_tracks());
}

/// A newer tick is prepared and dropped before the one in flight lands. The
/// install made before that preparation is still reconciled by the landing.
#[test]
fn an_edit_before_a_dropped_newer_preparation_survives_the_older_commit() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let (prepared, displaced) = terminal_tick_in_flight(&world, &mut sampler);

    sampler.install(world.receipt(2, 50.0, None)).unwrap();
    drop(sampler.prepare_tick(10_001, world.presentation).unwrap());
    let receipt = sampler.commit_prepared(prepared.presented_for_certification());

    assert!(receipt.terminals().is_empty());
    assert!(!sampler.contains_track(displaced));
    assert!(sampler.has_active_tracks());
}

/// The owner's posture is not the tick's to land.
#[test]
fn reduced_motion_set_while_a_tick_is_in_flight_survives_its_commit() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let (prepared, _) = terminal_tick_in_flight(&world, &mut sampler);

    sampler.set_reduced_motion(UiPresentationReducedMotionPosture::Reduce);
    sampler.commit_prepared(prepared.presented_for_certification());

    assert_eq!(
        sampler.reduced_motion(),
        UiPresentationReducedMotionPosture::Reduce
    );
}

#[test]
fn a_track_rebound_away_while_a_tick_is_in_flight_stays_retired() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let (prepared, rebound) = terminal_tick_in_flight(&world, &mut sampler);

    assert!(sampler.retire_rebound_track(rebound));
    let receipt = sampler.commit_prepared(prepared.presented_for_certification());

    assert!(receipt.terminals().is_empty());
    assert!(!sampler.contains_track(rebound));
    assert!(!sampler.has_active_tracks());
}

fn settle(
    world: &World,
    identity: u64,
    predecessor_y: f32,
    successor_y: f32,
    retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
) -> crate::runtime::motion::UiMotionCommitReceipt {
    settle_target(
        world,
        world.target,
        identity,
        predecessor_y,
        successor_y,
        retarget,
    )
}

fn settle_target(
    world: &World,
    target: crate::runtime::motion::UiMotionTargetIdentity,
    identity: u64,
    predecessor_y: f32,
    successor_y: f32,
    retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
) -> crate::runtime::motion::UiMotionCommitReceipt {
    crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
        identity,
        target,
        world.presentation,
        Some([0.0, predecessor_y, 240.0, 269.0]),
        true,
        Some([0.0, successor_y, 240.0, 269.0]),
        true,
        crate::runtime::motion::UiMotionDeclaration::scroll_settle(120),
        retarget,
    )
}

fn sampled_y(receipt: &UiPresentationMotionSamplingReceipt) -> f32 {
    receipt.samples()[0].geometry().unwrap().components()[1]
}

/// A tick older than one already landed would move the tracks back to what
/// the host showed before, so it lands nothing and claims nothing.
#[test]
fn a_tick_prepared_before_a_landed_one_lands_nothing() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(settle(&world, 1, 0.0, -60.0, None))
        .unwrap();
    commit_tick(&mut sampler, 100, world.presentation);
    let older = sampler.prepare_tick(120, world.presentation).unwrap();
    let newer = sampler.prepare_tick(140, world.presentation).unwrap();
    let presented = sampled_y(newer.receipt());
    sampler.commit_prepared(newer.presented_for_certification());

    let stale = sampler.commit_prepared(older.presented_for_certification());

    assert!(stale.samples().is_empty());
    assert!(stale.terminals().is_empty());
    let current = sampler.certification_observation().3.unwrap();
    assert_eq!(current.geometry().unwrap().components()[1], presented);
}

/// A wheel notch lands between a tick's preparation and its presentation. The
/// frame in flight still reaches the screen, so the notch's retarget departs
/// from what that frame shows, not from the frame before it.
#[test]
fn a_retarget_installed_while_a_tick_is_in_flight_departs_from_the_frame_it_presents() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(settle(&world, 1, 0.0, -60.0, None))
        .unwrap();
    commit_tick(&mut sampler, 100, world.presentation);
    let before = sampled_y(&commit_tick(&mut sampler, 120, world.presentation));
    let prepared = sampler.prepare_tick(140, world.presentation).unwrap();
    let presented = sampled_y(prepared.receipt());
    assert!(presented < before, "the outgoing settle is moving down");

    let retarget = crate::runtime::motion::UiMotionRetargetDisposition::Install {
        predecessor: crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
    };
    sampler
        .install(settle(&world, 2, -60.0, -120.0, Some(retarget)))
        .unwrap();
    sampler.commit_prepared(prepared.presented_for_certification());
    let next = sampled_y(&commit_tick(&mut sampler, 160, world.presentation));

    assert!(
        next <= presented + 0.01,
        "the retarget rewound from {presented} to {next}, toward the earlier frame at {before}"
    );
    assert!(
        next < presented - 1.0,
        "the retarget held the presented frame at {presented} instead of moving on: {next}"
    );
}

/// A publication rebinds the surface while a tick is in flight. The tick still
/// reaches the screen, so the track has presented its sample, now read against
/// the rebound presentation. Keeping the state from before the tick recorded
/// the previous frame as displayed, and scrolled content drifted from the
/// scrollbar and left stale pixels behind.
#[test]
fn a_publication_rebind_while_a_tick_is_in_flight_keeps_the_sample_it_presents() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(settle(&world, 1, 0.0, -60.0, None))
        .unwrap();
    commit_tick(&mut sampler, 100, world.presentation);
    let before = sampled_y(&commit_tick(&mut sampler, 120, world.presentation));
    let prepared = sampler.prepare_tick(140, world.presentation).unwrap();
    let presented = sampled_y(prepared.receipt());
    assert!(presented < before, "the settle is moving down");

    let rebound = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(2),
    );
    sampler.rebind_published_presentation(world.target.semantic_surface(), rebound);
    let landed = sampler.commit_prepared(prepared.presented_for_certification());

    assert_eq!(
        sampled_y(&landed),
        presented,
        "the presented sample was dropped"
    );
    let current = sampler.certification_observation().3.unwrap();
    assert_eq!(current.presentation_basis(), rebound);
    assert_eq!(current.geometry().unwrap().components()[1], presented);
    let next = sampled_y(&commit_tick(&mut sampler, 160, rebound));
    assert!(
        next < presented - 1.0,
        "the next frame moved from {next} rather than on from the presented {presented}"
    );
}

/// The retarget that lands with a tick in flight departs from the frame that
/// tick put on screen, so that frame is its presented start. Left unpresented,
/// the Scroll settle could not see the frame the host showed, the displayed
/// pose fell behind it, and the next sample damaged the published geometry
/// instead of the place the content left.
#[test]
fn a_scroll_retarget_landing_with_a_tick_in_flight_is_on_screen_where_that_tick_left_it() {
    let world = World::new();
    let target = crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
        world.target.semantic_surface(),
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        81,
    );
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(settle_target(&world, target, 1, 0.0, -60.0, None))
        .unwrap();
    commit_tick(&mut sampler, 100, world.presentation);
    commit_tick(&mut sampler, 120, world.presentation);
    let prepared = sampler.prepare_tick(140, world.presentation).unwrap();
    let presented = prepared.receipt().samples()[0]
        .geometry()
        .unwrap()
        .components();

    let retarget = crate::runtime::motion::UiMotionRetargetDisposition::Install {
        predecessor: crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
    };
    sampler
        .install(settle_target(
            &world,
            target,
            2,
            -60.0,
            -120.0,
            Some(retarget),
        ))
        .unwrap();
    sampler.commit_prepared(prepared.presented_for_certification());

    assert_eq!(
        sampler
            .accepted_scroll_group_sample(target)
            .map(|sample| [sample.components()[0], sample.components()[1]]),
        Some([presented[0], presented[1]]),
        "the Scroll settle reads the frame the host shows"
    );
    let next = commit_tick(&mut sampler, 160, world.presentation);
    let everywhere = UiPresentationSampledClipGeometry::from_presented_components([
        -1000.0, -1000.0, 4000.0, 4000.0,
    ])
    .unwrap();
    let left = next.samples()[0].damage().clipped_to(everywhere)[0]
        .unwrap()
        .components();
    assert_eq!(
        left, presented,
        "the next sample damages where the content was"
    );
}

#[test]
fn a_prepared_tick_lands_only_on_the_surface_generation_it_was_prepared_for() {
    use crate::mounting::presentation::presented_surface_witness_for_certification as witness;
    use worth_ui_host_contract::{
        UiHostObservationPresentationBasis as Basis, UiHostPresentationEpoch,
        UiHostSurfaceIdentity, UiMountedFrameIdentity, UiSurfaceBindingGeneration,
    };
    let world = World::new();
    let at = world.presentation;
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(1, 0.0, None)).unwrap();
    for other in [
        Basis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            at.frame(),
            at.binding(),
            at.epoch(),
        ),
        Basis::new(
            at.host_surface(),
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            at.binding(),
            at.epoch(),
        ),
        Basis::new(
            at.host_surface(),
            at.frame(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            at.epoch(),
        ),
    ] {
        let detached = sampler.prepare_tick(40, at).unwrap();
        assert!(
            detached.into_presented(&witness(other)).is_none(),
            "a witness for another surface generation cannot land this tick"
        );
    }
    let sampled = sampler.prepare_tick(40, at).unwrap();
    assert!(!sampled.receipt().samples().is_empty());
    let UiPreparedMotionWork::NeedsPresentation(sampled) = sampled.into_work() else {
        panic!("a tick with samples needs a presented witness");
    };
    let later = Basis::new(
        at.host_surface(),
        at.frame(),
        at.binding(),
        UiHostPresentationEpoch::issued_by_host(9),
    );
    let presented = sampled
        .into_presented(&witness(later))
        .expect("the host showed this frame at a later epoch");
    let receipt = sampler.commit_prepared(presented);
    assert_eq!(receipt.samples()[0].presentation_basis(), later);
}
