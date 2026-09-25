use super::*;
use crate::mounting::presented_hit_index::UiPresentedHitIndex;
use crate::mounting::spatial_index::UiMountedSpatialBudget;

#[test]
fn closed_portal_motion_cannot_hide_its_stationary_anchor() {
    let ordinary = World::new();
    let portal = World {
        target: crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
            ordinary.target.semantic_surface(),
            ordinary.target.mounted_instance(),
            ordinary.target.owner_key(),
        ),
        presentation: ordinary.presentation,
        displayed: ordinary.displayed,
    };
    assert_ne!(ordinary.target, portal.target);
    let mut sampler = UiMountedMotionSampler::default();
    let mut index = UiPresentedHitIndex::default();
    let base = crate::mounting::UiPresentedHitTestRow::from_mounted(
        crate::mounting::UiMountedHitTestPresentation::for_test(
            ordinary.hit_test_row([20.0, 10.0, 24.0, 12.0]),
        ),
    );
    index.replace_base(base.mounted_instance(), Some(base));
    sampler.install(portal.exit_receipt(303)).unwrap();
    commit_tick(&mut sampler, 1, ordinary.presentation);
    commit_tick(&mut sampler, 500, ordinary.presentation);
    index.apply_motion(&sampler, ordinary.displayed, &[portal.target]);
    assert_eq!(
        at(&index, &ordinary, [30.0, 16.0]),
        [base.mounted_instance()]
    );
    assert!(sampler
        .current_sample_for_with_work(base.mounted_instance(), ordinary.presentation)
        .0
        .is_none());
    assert!(sampler
        .current_sample_for_target(portal.target, ordinary.presentation)
        .is_some());
    sampler.install(ordinary.exit_receipt(304)).unwrap();
    commit_tick(&mut sampler, 501, ordinary.presentation);
    index.apply_motion(&sampler, ordinary.displayed, &[ordinary.target]);
    assert!(at(&index, &ordinary, [30.0, 16.0]).is_empty());
}

#[test]
fn indexed_motion_preserves_baseline_clip_retained_versions_and_committed_only_updates() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let mut index = UiPresentedHitIndex::default();
    let base = crate::mounting::UiPresentedHitTestRow::from_mounted(
        crate::mounting::UiMountedHitTestPresentation::for_test(
            world.hit_test_row([20.0, 10.0, 24.0, 12.0]),
        ),
    );
    index.replace_base(base.mounted_instance(), Some(base));
    let retained = index.clone();
    let reserved = index.retained_structural_bytes();
    sampler.install(world.receipt(300, 0.0, None)).unwrap();
    let prepared = sampler.prepare_tick(1, world.presentation).unwrap();
    assert_eq!(at(&index, &world, [30.0, 16.0]), [base.mounted_instance()]);
    drop(prepared);
    assert_eq!(at(&index, &world, [30.0, 16.0]), [base.mounted_instance()]);
    commit_tick(&mut sampler, 1, world.presentation);
    index.apply_motion(&sampler, world.displayed, &[world.target]);
    // The entrance begins outside the retained viewport clip.
    assert!(at(&index, &world, [30.0, 16.0]).is_empty());
    commit_tick(&mut sampler, 500, world.presentation);
    let work = index.apply_motion(&sampler, world.displayed, &[world.target]);
    assert!(work.node_visits < 16);
    assert_eq!(at(&index, &world, [30.0, 16.0]), [base.mounted_instance()]);
    assert_eq!(index.retained_structural_bytes(), reserved);
    sampler.install(world.exit_receipt(301)).unwrap();
    commit_tick(&mut sampler, 501, world.presentation);
    index.apply_motion(&sampler, world.displayed, &[world.target]);
    assert!(at(&index, &world, [30.0, 16.0]).is_empty());
    assert_eq!(
        at(&retained, &world, [30.0, 16.0]),
        [base.mounted_instance()]
    );
    assert_eq!(index.retained_structural_bytes(), reserved);
    // A fresh visible baseline can be restored after an explicit mounted change.
    let other = World::new();
    index.replace_base(base.mounted_instance(), None);
    assert_eq!(index.retained_structural_bytes(), Some(0));
    assert!(at(&retained, &other, [30.0, 16.0]).is_empty());
}

#[test]
fn transparent_motion_does_not_remove_an_otherwise_visible_hit_target() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(
            crate::runtime::motion::UiMotionCommitReceipt::for_sampling_test_transition(
                302,
                world.target,
                world.presentation,
                Some([20.0, 10.0, 24.0, 12.0]),
                false,
                Some([20.0, 10.0, 24.0, 12.0]),
                true,
                crate::runtime::motion::UiMotionDeclaration::portal_entrance(),
                None,
            ),
        )
        .unwrap();
    let receipt = commit_tick(&mut sampler, 1, world.presentation);
    let sample = receipt.samples()[0];
    assert_eq!(sample.opacity_units(), 0);
    assert!(
        sample.hit_test_visible(),
        "opacity is not interaction authority"
    );
}

fn at(
    index: &UiPresentedHitIndex,
    world: &World,
    point: [f64; 2],
) -> Vec<worth_ui_host_contract::UiMountedInstanceIdentity> {
    index
        .at_point(
            world.presentation.binding(),
            point,
            UiMountedSpatialBudget {
                node_visits: 64,
                candidates: 16,
            },
        )
        .unwrap()
        .rows
        .iter()
        .map(|row| row.mounted_instance())
        .collect()
}
