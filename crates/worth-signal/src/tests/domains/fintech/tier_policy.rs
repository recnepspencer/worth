use crate::facade::*;

use super::execution_tier::FintechTier;
use super::scales::FintechScale;
use super::scenarios::setup_seeded_world;

#[test]
fn fintech_mixed_tier_policy_honors_audit_tolerance_without_hiding_live_truth_changes() {
    let mut world = setup_seeded_world();
    world.assert_shape(FintechScale::smoke());

    world
        .runtime
        .set_node_tier(world.top_desk(), FintechTier::Audit);
    world
        .runtime
        .set_node_tier(world.primary_threshold_node(), FintechTier::Live);
    world.runtime.set_tier_policy(
        TierPolicy::new(
            FintechTier::Audit,
            DependencyMode::AutoDiscovered,
            DirtyPropagation::Immediate,
            EvaluationTrigger::LazyPull,
        )
        .with_default_comparator(VersionComparatorPolicy::Tolerance { epsilon: 64 }),
    );
    world.runtime.set_tier_policy(
        TierPolicy::new(
            FintechTier::Live,
            DependencyMode::AutoDiscovered,
            DirtyPropagation::Immediate,
            EvaluationTrigger::LazyPull,
        )
        .with_default_comparator(VersionComparatorPolicy::Exact),
    );

    let baseline_desk = world.read_top_desk().unwrap();
    let baseline_threshold = world.read_primary_threshold().unwrap();
    let skipped_before = world
        .runtime
        .graph()
        .observe()
        .metrics()
        .evaluation
        .skipped_by_comparator;

    world.bump_primary_market(3, 0, 0, 0).unwrap();

    let after_desk = world.read_top_desk().unwrap();
    let after_threshold = world.read_primary_threshold().unwrap();

    assert_eq!(after_desk, baseline_desk);
    assert_ne!(after_threshold, baseline_threshold);
    assert!(
        world
            .runtime
            .graph()
            .observe()
            .metrics()
            .evaluation
            .skipped_by_comparator
            > skipped_before
    );
}
