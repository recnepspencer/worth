//! Velocity continuity across a Motion retarget, and the structural
//! distinctness of the three target scopes that share a mounted instance.

use super::*;

const SETTLE_TICKS: u32 = 120;
const TOLERANCE: f64 = 2.0e-3;

/// A notch arriving mid-settle must leave the content moving at the rate it was
/// already moving. The successor curve is fully determined by the interrupted
/// sample's position, that rate, the new endpoint and the settle horizon, so
/// asserting every later sample against an independently written Hermite curve
/// proves position *and* velocity continuity at once. The curve's clock runs
/// from the interrupted sample, which the screen already shows: the successor's
/// first frame moves on from it rather than holding it for a frame.
#[test]
fn a_retarget_at_one_third_of_the_horizon_departs_at_the_interrupted_rate() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.settle(1, 0.0, -60.0, None)).unwrap();

    commit_at(&mut sampler, 100, &world);
    let interruption_tick = 100 + u64::from(SETTLE_TICKS) / 3;
    let interrupted = translation_y(&commit_at(&mut sampler, interruption_tick, &world));

    let elapsed = f64::from(SETTLE_TICKS) / 3.0;
    assert_close(
        interrupted,
        hermite_position(0.0, 0.0, -60.0, elapsed, f64::from(SETTLE_TICKS)),
        "the interrupted sample sits on the outgoing curve",
    );
    let departure_rate = hermite_rate(0.0, 0.0, -60.0, elapsed, f64::from(SETTLE_TICKS));
    assert!(
        departure_rate < -0.1,
        "the outgoing curve is genuinely moving at the interruption, rate {departure_rate}"
    );

    let installed = sampler
        .install(world.settle(2, -60.0, -120.0, Some(retarget_from_current_sample())))
        .unwrap();
    assert_close(
        f64::from(installed.sample().unwrap().geometry().unwrap().components()[1]),
        interrupted,
        "the successor begins at the accepted sample's position",
    );

    for offset in [1_u64, 2, 20, 60, 119, u64::from(SETTLE_TICKS)] {
        let sampled = translation_y(&commit_at(&mut sampler, interruption_tick + offset, &world));
        assert_close(
            sampled,
            hermite_position(
                interrupted,
                departure_rate,
                -120.0,
                offset as f64,
                f64::from(SETTLE_TICKS),
            ),
            "the successor follows the velocity-matched curve from the interrupted rate",
        );
    }
}

/// The same scenario with the carried rate forced to zero is a different curve.
/// Without this, the continuity assertions above would also pass for a sampler
/// that reset velocity at every retarget.
#[test]
fn resetting_the_carried_rate_to_zero_would_produce_a_visibly_different_curve() {
    let elapsed = f64::from(SETTLE_TICKS) / 3.0;
    let interrupted = hermite_position(0.0, 0.0, -60.0, elapsed, f64::from(SETTLE_TICKS));
    let departure_rate = hermite_rate(0.0, 0.0, -60.0, elapsed, f64::from(SETTLE_TICKS));

    let continuous = hermite_position(
        interrupted,
        departure_rate,
        -120.0,
        19.0,
        f64::from(SETTLE_TICKS),
    );
    let reset = hermite_position(interrupted, 0.0, -120.0, 19.0, f64::from(SETTLE_TICKS));

    assert!(
        (continuous - reset).abs() > 100.0 * TOLERANCE,
        "a zero-velocity restart is separated from continuity by far more than tolerance: \
         continuous {continuous}, reset {reset}"
    );
}

/// The declared settle family arrives exactly on its endpoint and arrives at
/// rest, so settlement neither undershoots nor stops abruptly.
#[test]
fn velocity_matched_settlement_reaches_its_endpoint_at_rest() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.settle(3, 0.0, -120.0, None)).unwrap();

    commit_at(&mut sampler, 10, &world);
    let mut sampled = Vec::new();
    for elapsed in [58_u64, 59, 118, 119, u64::from(SETTLE_TICKS)] {
        let receipt = commit_at(&mut sampler, 10 + elapsed, &world);
        assert_close(
            translation_y(&receipt),
            hermite_position(0.0, 0.0, -120.0, elapsed as f64, f64::from(SETTLE_TICKS)),
            "settlement follows the velocity-matched curve",
        );
        sampled.push((translation_y(&receipt), receipt.samples()[0].posture()));
    }

    assert_close(
        sampled[4].0,
        -120.0,
        "settlement ends exactly on the declared endpoint",
    );
    assert_eq!(sampled[4].1, UiPresentationMotionSamplePosture::Terminal);

    let mid_step = (sampled[1].0 - sampled[0].0).abs();
    let final_step = (sampled[3].0 - sampled[2].0).abs();
    assert!(
        mid_step > 1.0,
        "the curve is genuinely moving mid-horizon, step {mid_step}"
    );
    assert!(
        final_step < mid_step / 10.0,
        "the final per-tick displacement decays toward rest: final {final_step}, mid {mid_step}"
    );
}

/// The fixed-shape family predates this work and must sample exactly as before:
/// one eased scalar applied uniformly, with no carried velocity anywhere in it.
#[test]
fn ease_out_cubic_tracks_still_sample_their_own_fixed_shape() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(world.portal_entrance(4, 0.0, 40.0))
        .unwrap();

    commit_at(&mut sampler, 200, &world);
    for elapsed in [1_u64, 35, 70, 139] {
        let sampled = translation_y(&commit_at(&mut sampler, 200 + elapsed, &world));
        let progress = elapsed as f64 / 140.0;
        assert_close(
            sampled,
            40.0 * (1.0 - (1.0 - progress).powi(3)),
            "the fixed-shape ease is unchanged",
        );
    }
}

/// An ordinary component, its Portal content and its scrolled content are three
/// different moving things bound to the same mounted instance. They must remain
/// three keys, or one of them silently displaces another's track.
#[test]
fn the_three_target_scopes_of_one_owner_are_distinct_keys() {
    let world = World::new();
    let ordinary = crate::runtime::motion::UiMotionTargetIdentity::from_mounted_owner(
        world.semantic,
        world.instance,
        7,
    );
    let portal = crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
        world.semantic,
        world.instance,
        7,
    );
    let scrolled = crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
        world.semantic,
        world.instance,
        7,
    );

    assert_eq!(
        ordinary.scope(),
        crate::runtime::motion::UiMotionTargetScope::Ordinary
    );
    assert_eq!(
        portal.scope(),
        crate::runtime::motion::UiMotionTargetScope::PortalContents
    );
    assert_eq!(
        scrolled.scope(),
        crate::runtime::motion::UiMotionTargetScope::ScrollContents
    );
    assert!(portal.is_portal_contents());
    assert!(!ordinary.is_portal_contents());
    assert!(!scrolled.is_portal_contents());
    assert_eq!(
        std::collections::BTreeSet::from([ordinary, portal, scrolled]).len(),
        3
    );

    let mut sampler = UiMountedMotionSampler::default();
    for (identity, target) in [(11, ordinary), (12, portal), (13, scrolled)] {
        sampler
            .install(world.settle_for_target(identity, target, 0.0, -30.0, None))
            .unwrap();
    }
    let observation = sampler.certification_observation();
    assert_eq!(observation.1, 3, "each scope retains its own track");

    commit_at(&mut sampler, 5, &world);
    let accepted = sampler
        .accepted_scroll_group_translation(scrolled)
        .expect("an accepted Scroll-content sample reports its group translation");
    assert_close(f64::from(accepted[1]), 0.0, "the settle has just begun");
    assert_eq!(sampler.accepted_scroll_group_translation(ordinary), None);
    assert_eq!(sampler.accepted_scroll_group_translation(portal), None);
}

/// Cubic Hermite from `(start, start_rate)` to `(end, at rest)` over
/// `duration` ticks, written here from the Hermite basis functions so the
/// production curve is never its own oracle.
fn hermite_position(start: f64, start_rate: f64, end: f64, elapsed: f64, duration: f64) -> f64 {
    let s = (elapsed / duration).clamp(0.0, 1.0);
    let from_start = 2.0 * s.powi(3) - 3.0 * s.powi(2) + 1.0;
    let from_start_rate = s.powi(3) - 2.0 * s.powi(2) + s;
    let from_end = -2.0 * s.powi(3) + 3.0 * s.powi(2);
    from_start * start + from_start_rate * duration * start_rate + from_end * end
}

/// The per-tick derivative of [`hermite_position`], from the derivatives of the
/// same four basis functions.
fn hermite_rate(start: f64, start_rate: f64, end: f64, elapsed: f64, duration: f64) -> f64 {
    let s = (elapsed / duration).clamp(0.0, 1.0);
    let from_start = 6.0 * s.powi(2) - 6.0 * s;
    let from_start_rate = 3.0 * s.powi(2) - 4.0 * s + 1.0;
    let from_end = -6.0 * s.powi(2) + 6.0 * s;
    (from_start * start + from_end * end) / duration + from_start_rate * start_rate
}

fn assert_close(observed: f64, expected: f64, claim: &str) {
    assert!(
        (observed - expected).abs() <= TOLERANCE,
        "{claim}: observed {observed}, expected {expected}"
    );
}

fn translation_y(receipt: &UiPresentationMotionSamplingReceipt) -> f64 {
    f64::from(receipt.samples()[0].geometry().unwrap().components()[1])
}

fn commit_at(
    sampler: &mut UiMountedMotionSampler,
    tick: u64,
    world: &World,
) -> UiPresentationMotionSamplingReceipt {
    let prepared = sampler.prepare_tick(tick, world.presentation).unwrap();
    sampler.commit_prepared(prepared.presented_for_certification())
}

fn retarget_from_current_sample() -> crate::runtime::motion::UiMotionRetargetDisposition {
    crate::runtime::motion::UiMotionRetargetDisposition::Install {
        predecessor: crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
    }
}

struct World {
    semantic: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    scroll_target: crate::runtime::motion::UiMotionTargetIdentity,
    ordinary_target: crate::runtime::motion::UiMotionTargetIdentity,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

impl World {
    fn new() -> Self {
        let semantic = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        Self {
            semantic,
            instance,
            scroll_target: crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
                semantic, instance, 31,
            ),
            ordinary_target: crate::runtime::motion::UiMotionTargetIdentity::from_mounted_owner(
                semantic, instance, 31,
            ),
            presentation: worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
                worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
                worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
            ),
        }
    }

    fn settle(
        &self,
        identity: u64,
        predecessor_y: f32,
        successor_y: f32,
        retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        self.settle_for_target(
            identity,
            self.scroll_target,
            predecessor_y,
            successor_y,
            retarget,
        )
    }

    fn settle_for_target(
        &self,
        identity: u64,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        predecessor_y: f32,
        successor_y: f32,
        retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            target,
            self.presentation,
            Some([0.0, predecessor_y, 240.0, 269.0]),
            true,
            Some([0.0, successor_y, 240.0, 269.0]),
            true,
            crate::runtime::motion::UiMotionDeclaration::scroll_settle(SETTLE_TICKS),
            retarget,
        )
    }

    fn portal_entrance(
        &self,
        identity: u64,
        predecessor_y: f32,
        successor_y: f32,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            self.ordinary_target,
            self.presentation,
            Some([0.0, predecessor_y, 240.0, 269.0]),
            true,
            Some([0.0, successor_y, 240.0, 269.0]),
            true,
            crate::runtime::motion::UiMotionDeclaration::portal_entrance(),
            None,
        )
    }
}
