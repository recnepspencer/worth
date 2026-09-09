use super::*;

#[path = "tests/damage.rs"]
mod damage;
#[path = "tests/opacity.rs"]
mod opacity;
#[path = "tests/presented_index.rs"]
mod presented_index;

fn commit_tick(
    sampler: &mut UiMountedMotionSampler,
    tick: u64,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
) -> UiPresentationMotionSamplingReceipt {
    let prepared = sampler.prepare_tick(tick, presentation).unwrap();
    sampler.commit_prepared(prepared)
}

#[test]
fn inactive_sampling_has_zero_work_and_rejects_a_repeated_tick() {
    let mut sampler = UiMountedMotionSampler::default();
    let world = World::new();
    let receipt = commit_tick(&mut sampler, 1, world.presentation);
    assert_eq!(receipt.cost(), UiPresentationMotionSamplingCost::default());
    assert!(!sampler.has_active_tracks());
    assert!(matches!(
        sampler.prepare_tick(1, world.presentation),
        Err(UiPresentationMotionSamplingDenial::NonMonotonicTick)
    ));
    let observation = sampler.certification_observation();
    assert_eq!(observation.4, 1);
    assert_eq!(
        observation.5,
        Some(UiPresentationMotionSamplingDenial::NonMonotonicTick)
    );
}

#[test]
fn retargeting_continues_from_the_current_presentation_sample() {
    let world = World::new();
    let mut current_sampler = UiMountedMotionSampler::default();
    current_sampler
        .install(world.receipt(1, 0.0, None))
        .unwrap();
    let mid = commit_tick(&mut current_sampler, 70, world.presentation).samples()[0]
        .geometry()
        .unwrap()
        .components();
    let current = world.receipt(
        2,
        100.0,
        Some(
            crate::runtime::motion::UiMotionRetargetDisposition::Install {
                predecessor:
                    crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
            },
        ),
    );
    let installed = current_sampler.install(current).unwrap();
    assert_eq!(
        installed.sample().unwrap().geometry().unwrap().components(),
        mid
    );
}

#[test]
fn reduced_motion_snaps_decorative_motion_and_shortens_necessary_motion() {
    let world = World::new();
    let mut decorative = UiMountedMotionSampler::default();
    decorative.set_reduced_motion(UiPresentationReducedMotionPosture::Reduce);
    let snapped = decorative.install(world.receipt(40, 0.0, None)).unwrap();
    assert_eq!(
        snapped.terminal().unwrap().cause(),
        crate::runtime::motion::UiMotionTerminalCause::SnappedToTarget
    );

    let mut necessary = UiMountedMotionSampler::default();
    necessary.set_reduced_motion(UiPresentationReducedMotionPosture::Reduce);
    necessary
        .install(world.receipt_with_declaration(
            41,
            0.0,
            crate::runtime::motion::UiMotionDeclaration::rebind_geometry(),
            None,
        ))
        .unwrap();
    commit_tick(&mut necessary, 8, world.presentation);
    let terminal = commit_tick(&mut necessary, 9, world.presentation);
    assert_eq!(terminal.terminals().len(), 1);
    assert!(!necessary.has_active_tracks());
}

#[test]
fn presented_hit_rows_use_only_committed_geometry_and_hide_a_presented_exit() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(50, 0.0, None)).unwrap();
    commit_tick(&mut sampler, 1, world.presentation);
    commit_tick(&mut sampler, 71, world.presentation);
    let mounted = world.hit_test_row([20.0, 10.0, 24.0, 12.0]);
    let mut basis = crate::mounting::UiPresentedHitTestBasis::new(
        world.presentation,
        crate::mounting::UiPresentedFrameBasisRelation::Current,
        vec![crate::mounting::UiMountedHitTestPresentation::for_test(
            mounted,
        )]
        .into_boxed_slice(),
    );
    basis.apply_motion_samples(&sampler);
    let sampled = basis.rows()[0];
    let sampled_y = sampled.bounds().y();
    assert!(sampled_y > 0.0 && sampled_y < 10.0);
    assert_eq!(
        sampled.bounds().coordinate_space(),
        mounted.bounds().coordinate_space()
    );
    assert_eq!(
        sampled.clip_bounds(),
        mounted.clip_bounds(),
        "motion changes the presented target bounds, not the viewport scissor"
    );

    let exit = world.exit_receipt(51);
    let exit_track = exit.track().identity();
    sampler.install(exit).unwrap();
    let mut before_exit_sample = crate::mounting::UiPresentedHitTestBasis::new(
        world.presentation,
        crate::mounting::UiPresentedFrameBasisRelation::Current,
        vec![crate::mounting::UiMountedHitTestPresentation::for_test(
            mounted,
        )]
        .into_boxed_slice(),
    );
    before_exit_sample.apply_motion_samples(&sampler);
    assert_eq!(before_exit_sample.rows().len(), 1);

    commit_tick(&mut sampler, 72, world.presentation);
    let mut exiting = crate::mounting::UiPresentedHitTestBasis::new(
        world.presentation,
        crate::mounting::UiPresentedFrameBasisRelation::Current,
        vec![crate::mounting::UiMountedHitTestPresentation::for_test(
            mounted,
        )]
        .into_boxed_slice(),
    );
    exiting.apply_motion_samples(&sampler);
    assert!(exiting.rows().is_empty());

    let terminal = commit_tick(&mut sampler, 182, world.presentation);
    assert_eq!(terminal.terminals().len(), 1);
    assert!(!sampler.has_active_tracks());
    assert!(sampler.retire_terminal_track(exit_track));
    let mut after_portal_terminal = crate::mounting::UiPresentedHitTestBasis::new(
        world.presentation,
        crate::mounting::UiPresentedFrameBasisRelation::Current,
        vec![crate::mounting::UiMountedHitTestPresentation::for_test(
            mounted,
        )]
        .into_boxed_slice(),
    );
    after_portal_terminal.apply_motion_samples(&sampler);
    assert_eq!(after_portal_terminal.rows().len(), 1);
}

#[test]
fn discarded_prepared_sample_changes_no_presented_truth_or_terminal_state() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(70, 0.0, None)).unwrap();
    let before = sampler.certification_observation();

    let prepared = sampler.prepare_tick(200, world.presentation).unwrap();
    drop(prepared);

    let after = sampler.certification_observation();
    assert_eq!(after.0, before.0);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, before.3);
    assert!(sampler.has_active_tracks());
}

#[test]
fn active_track_capacity_denies_before_mutating_the_sampler() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    for identity in 1..=sampling::MAX_PRESENTATION_TRACKS as u64 {
        let target = crate::runtime::motion::UiMotionTargetIdentity::from_family_owner(
            world.target.semantic_surface(),
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
            identity,
        );
        sampler
            .install(world.receipt_for_target(identity, target))
            .unwrap();
    }
    let before = sampler.certification_observation();
    let overflow = crate::runtime::motion::UiMotionTargetIdentity::from_family_owner(
        world.target.semantic_surface(),
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        10_000,
    );

    assert_eq!(
        sampler.install(world.receipt_for_target(10_000, overflow)),
        Err(UiPresentationMotionSamplingDenial::TrackCapacityExceeded)
    );
    let after = sampler.certification_observation();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, before.3);
    assert_eq!(after.4, before.4 + 1);
    assert_eq!(
        after.5,
        Some(UiPresentationMotionSamplingDenial::TrackCapacityExceeded)
    );
}

#[test]
fn published_surface_rebind_preserves_the_active_track_and_current_sample() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(75, 0.0, None)).unwrap();
    let mid = commit_tick(&mut sampler, 70, world.presentation).samples()[0];
    let replacement = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(2),
    );

    sampler.rebind_published_presentation(world.target.semantic_surface(), replacement);

    let current = sampler.certification_observation().3.unwrap();
    assert_eq!(current.presentation_basis(), replacement);
    assert_eq!(current.opacity_units(), mid.opacity_units());
    assert_eq!(
        current.geometry().unwrap().presentation_basis(),
        replacement
    );
    let next = commit_tick(&mut sampler, 71, replacement);
    assert_eq!(next.samples().len(), 1);
    assert!(next.terminals().is_empty());
}

#[test]
fn a_track_installed_after_a_long_idle_starts_on_its_own_first_tick() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(80, 0.0, None)).unwrap();
    commit_tick(&mut sampler, 1, world.presentation);
    let first_terminal = commit_tick(&mut sampler, 141, world.presentation);
    assert_eq!(first_terminal.terminals().len(), 1);
    assert!(!sampler.has_active_tracks());

    sampler.install(world.receipt(81, 100.0, None)).unwrap();
    let after_idle = commit_tick(&mut sampler, 10_141, world.presentation);
    assert!(after_idle.terminals().is_empty());
    assert_eq!(
        after_idle.samples()[0].posture(),
        UiPresentationMotionSamplePosture::Active
    );
    assert_eq!(
        after_idle.samples()[0].geometry().unwrap().components()[1],
        0.0
    );
    assert!(sampler.has_active_tracks());
}

struct World {
    target: crate::runtime::motion::UiMotionTargetIdentity,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

impl World {
    fn new() -> Self {
        let semantic = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let host = worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap();
        let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        Self {
            target: crate::runtime::motion::UiMotionTargetIdentity::from_family_owner(
                semantic,
                worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
                7,
            ),
            presentation: worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                host,
                frame,
                binding,
                worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
            ),
        }
    }

    fn receipt(
        &self,
        identity: u64,
        predecessor_x: f32,
        retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        self.receipt_with_declaration(
            identity,
            predecessor_x,
            crate::runtime::motion::UiMotionDeclaration::portal_entrance(),
            retarget,
        )
    }

    fn receipt_with_declaration(
        &self,
        identity: u64,
        predecessor_x: f32,
        declaration: crate::runtime::motion::UiMotionDeclaration,
        retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            self.target,
            self.presentation,
            Some([predecessor_x, 0.0, 20.0, 10.0]),
            true,
            Some([predecessor_x + 20.0, 10.0, 24.0, 12.0]),
            true,
            declaration,
            retarget,
        )
    }

    fn receipt_for_target(
        &self,
        identity: u64,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> crate::runtime::motion::UiMotionCommitReceipt {
        crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            target,
            self.presentation,
            Some([0.0, 0.0, 20.0, 10.0]),
            true,
            Some([20.0, 10.0, 24.0, 12.0]),
            true,
            crate::runtime::motion::UiMotionDeclaration::portal_entrance(),
            None,
        )
    }

    fn exit_receipt(&self, identity: u64) -> crate::runtime::motion::UiMotionCommitReceipt {
        crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
            identity,
            self.target,
            self.presentation,
            Some([20.0, 10.0, 24.0, 12.0]),
            true,
            Some([20.0, 10.0, 24.0, 12.0]),
            false,
            crate::runtime::motion::UiMotionDeclaration::portal_exit(),
            None,
        )
    }

    fn hit_test_row(
        &self,
        components: [f32; 4],
    ) -> worth_ui_host_contract::UiMountedHitTestMechanic {
        crate::mounting::retention::motion_sampling_hit_test_mechanic_for_test(
            self.presentation,
            self.target,
            components,
        )
    }
}
