use super::super::{computation, seeded_world, structural_cost};
use super::*;
use crate::checkpoint_recovery::checkpoint_recovery_test_guard;
fn assert_committed_facts(observed: &Observed, expected: &History) {
    for &(key, value, applied) in &observed.committed_facts {
        let encoded = expected
            .effects
            .iter()
            .find(|(identity, _)| *identity == key)
            .map(|(_, value)| *value);
        assert_eq!(
            applied,
            u64::from(encoded.is_some()),
            "committed application count for {key}"
        );
        assert_eq!(
            value,
            encoded.map(u64::from_le_bytes).unwrap_or(0),
            "committed semantic value for {key}"
        );
    }
}
#[test]
fn serial_seed_per_family_matches_independent_history() {
    let _guard = checkpoint_recovery_test_guard();
    for family in FAMILIES {
        let world = seeded_world::world(family, SEED);
        let expected = world.expected(
            OrderRule::CallerTraversal,
            FailureChargeRule::COMPLETED_MEMBERS_ONLY,
        );
        let observed = observe(&world, Posture::Serial);
        assert_eq!(
            observed.failure, expected.failure,
            "{family:?}: modeled failure"
        );
        assert_eq!(
            observed.kernel_work, expected.kernel_work,
            "{family:?}: settled kernel work"
        );
        assert_eq!(
            observed.committed,
            expected
                .effects
                .iter()
                .map(|(key, _)| *key)
                .collect::<Vec<_>>(),
            "{family:?}: owner-recorded semantic commit sequence"
        );
        assert_eq!(
            observed.computation_work,
            declared_charges(&world, &expected),
            "{family:?}: exact managed charges"
        );
        assert_committed_facts(&observed, &expected);
    }
}
fn declared_charges(world: &World, history: &History) -> Vec<u64> {
    world
        .order(OrderRule::CallerTraversal)
        .iter()
        .filter_map(|index| {
            let member = &world.members[*index];
            history
                .effects
                .iter()
                .any(|(key, _)| *key == member.key)
                .then(|| {
                    let keys = computation::declared_key_work(&member.partitions);
                    let kernels = member.partitions.iter().map(|p| p.work).sum::<u64>();
                    keys + structural_cost::routing_work(member.partitions.len() as u64)
                        + kernels
                        + structural_cost::reduction_work(&computation::identities(
                            &member.partitions,
                        ))
                })
        })
        .collect()
}
#[test]
fn declared_failure_at_each_member_position_stops_before_its_effect() {
    let _guard = checkpoint_recovery_test_guard();
    for position in 0..4 {
        for rank in [0, 1, 3] {
            let mut world = seeded_world::world(seeded_world::Family::Nested, SEED);
            let index = world.order(OrderRule::CallerTraversal)[position];
            world.members[index].partitions[rank].fails = true;
            world.members[index].partitions[3].fails = true;
            let expected = world.expected(
                OrderRule::CallerTraversal,
                FailureChargeRule::COMPLETED_MEMBERS_ONLY,
            );
            let observed = observe(&world, Posture::Serial);
            assert_eq!(
                observed.failure, expected.failure,
                "least failing partition from the model, member {position}, rank {rank}"
            );
            assert_eq!(
                observed.kernel_work, expected.kernel_work,
                "completed member charge prefix"
            );
            assert_eq!(
                observed.computation_work,
                declared_charges(&world, &expected),
                "completed managed charges"
            );
            assert_committed_facts(&observed, &expected);
        }
    }
}

#[test]
fn seeded_public_histories_agree_at_serial_one_two_and_four_workers() {
    let _guard = checkpoint_recovery_test_guard();
    for family in FAMILIES {
        let world = seeded_world::world(family, SEED);
        let expected = world.expected(
            OrderRule::CallerTraversal,
            FailureChargeRule::COMPLETED_MEMBERS_ONLY,
        );
        let serial = observe(&world, Posture::Serial);
        for width in [1, 2, 4] {
            let observed = observe(
                &world,
                Posture::Leased {
                    workers: std::num::NonZeroUsize::new(width).unwrap(),
                },
            );
            assert_eq!(observed.failure, expected.failure);
            assert_eq!(observed.kernel_work, expected.kernel_work);
            assert_eq!(observed.committed_facts, serial.committed_facts);
            assert_eq!(
                observed.computation_work,
                declared_charges(&world, &expected)
            );
            assert_eq!(
                observed.committed,
                expected
                    .effects
                    .iter()
                    .map(|(key, _)| *key)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                observed
                    .advancements
                    .iter()
                    .map(|r| (r.report.charged_work(), r.report.charged_span()))
                    .collect::<Vec<_>>(),
                serial
                    .advancements
                    .iter()
                    .map(|r| (r.report.charged_work(), r.report.charged_span()))
                    .collect::<Vec<_>>(),
                "{family:?} width {width}: structural advancement cost"
            );
            assert_committed_facts(&observed, &expected);
        }
    }
}
