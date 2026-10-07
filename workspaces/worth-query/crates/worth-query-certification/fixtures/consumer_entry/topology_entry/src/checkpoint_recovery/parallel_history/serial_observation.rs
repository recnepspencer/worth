//! Serial public requests and committed rows; part two replaces the placement and
//! report adapter with one advancement report and canonical committed identities.
use super::{
    application::Program,
    expected_history::{FailureChargeRule, History, OrderRule, World},
    installation, operation, read,
    seeded_world::{FAMILIES, SEED},
};
use std::collections::BTreeSet;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    partitioned_computation_runs_on_this_thread_for_test as runs,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
#[derive(Clone, Copy)]
pub(super) enum Posture {
    Serial,
}
pub(super) struct AdvancementReport {
    pub managed_charges: Vec<u64>,
}
// Ordered commit positions wait for the advancement report in part two.
pub(super) struct Observed {
    pub failure: Option<(u64, u64)>,
    pub kernel_work: u64,
    pub report: AdvancementReport,
    pub committed_facts: Vec<(u64, u64, u64)>,
}
pub(super) fn observe(world: &World, posture: Posture) -> Observed {
    assert!(matches!(posture, Posture::Serial));
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _restore = Restore(place(Placement::Serial));
    let application = installation::install(world);
    let (scope, principal) = installation::authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut visited = BTreeSet::new();
    let mut stack: Vec<_> = world
        .requested
        .iter()
        .rev()
        .map(|index| (*index, false))
        .collect();
    let mut failure = None;
    let mut kernel_work = 0;
    operation::CHARGES.with(|charges| charges.borrow_mut().clear());
    runs();
    while let Some((index, ready)) = stack.pop() {
        if visited.contains(&index) {
            continue;
        }
        let member = &world.members[index];
        if !ready {
            stack.push((index, true));
            stack.extend(member.upstream.iter().rev().map(|index| (*index, false)));
            continue;
        }
        let input = operation::Input {
            key: member.key,
            upstream: member
                .upstream
                .iter()
                .map(|index| world.members[*index].key)
                .collect(),
            partitions: member.partitions.clone(),
        };
        let outcome = request
            .mutate(input)
            .idempotency(&member.key)
            .execute_in_program::<Program>(&application)
            .unwrap();
        match outcome {
            Outcome::Committed { result, .. } => {
                let rows = request
                    .query(read::Read { key: member.key })
                    .execute()
                    .unwrap();
                let value = &rows.rows()[0];
                assert_eq!(value.applied, 1, "each member effect applied once");
                assert_eq!(
                    value.value, result.value,
                    "mutation result agrees with the committed semantic value"
                );
                visited.insert(index);
            }
            Outcome::DomainDenied(denied) => {
                assert!(runs().is_empty(), "failed members have no completed run");
                failure = Some((member.key, denied.partition));
                break;
            }
            other => panic!("unexpected member outcome: {other:?}"),
        }
        let reports = runs();
        use worth_query_host::facade::application_contribution::{
            WorthQueryPartitionedComputationFullCause as Cause,
            WorthQueryPartitionedComputationRun as Run,
        };
        let [(Run::Full(Cause::NoPriorRecord), Some(report))] = reports.as_slice() else {
            panic!("exactly one full computation run per completed member: {reports:?}");
        };
        let reduction = super::structural_cost::reduction_work(&super::computation::identities(
            &member.partitions,
        ));
        kernel_work += report.charged_work() - reduction;
    }
    let committed_facts = world
        .members
        .iter()
        .map(|member| {
            let rows = request
                .query(read::Read { key: member.key })
                .execute()
                .unwrap();
            let value = &rows.rows()[0];
            (member.key, value.value, value.applied)
        })
        .collect();
    let managed_charges =
        operation::CHARGES.with(|charges| std::mem::take(&mut *charges.borrow_mut()));
    Observed {
        failure,
        kernel_work,
        report: AdvancementReport { managed_charges },
        committed_facts,
    }
}
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
    let _guard = super::super::checkpoint_recovery_test_guard();
    for family in FAMILIES {
        let world = super::seeded_world::world(family, SEED);
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
        // Rows prove values and application counts. Request traversal is not
        // evidence of system commit order; that assertion waits for 7.7.
        assert_eq!(
            observed.report.managed_charges,
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
                    let keys = super::computation::declared_key_work(&member.partitions);
                    let kernels = member.partitions.iter().map(|p| p.work).sum::<u64>();
                    keys + super::structural_cost::routing_work(member.partitions.len() as u64)
                        + kernels
                        + super::structural_cost::reduction_work(&super::computation::identities(
                            &member.partitions,
                        ))
                })
        })
        .collect()
}
#[test]
fn declared_failure_at_each_member_position_stops_before_its_effect() {
    let _guard = super::super::checkpoint_recovery_test_guard();
    for position in 0..4 {
        for rank in [0, 1, 3] {
            let mut world = super::seeded_world::world(super::seeded_world::Family::Nested, SEED);
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
                observed.report.managed_charges,
                declared_charges(&world, &expected),
                "completed managed charges"
            );
            assert_committed_facts(&observed, &expected);
        }
    }
}
