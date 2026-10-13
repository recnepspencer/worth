//! Public request histories read owner commit evidence from the advancement observer.
//! A required-wave history plugs into the same report/identity boundary.
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
    advancement_observations_on_this_thread_for_test as advancements,
    partitioned_computation_runs_on_this_thread_for_test as runs,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementEventKindForTest as EventKind,
    WorthQueryAdvancementIdentityForTest as AppliedIdentity,
    WorthQueryExecutionPlacementForTest as Placement,
};
#[derive(Clone, Copy)]
pub(super) enum Posture {
    Serial,
    Leased { workers: std::num::NonZeroUsize },
}
pub(super) struct Observed {
    pub failure: Option<(u64, u64)>,
    pub kernel_work: u64,
    pub advancements:
        Vec<worth_query_host::facade::primary_graph::WorthQueryAdvancementObservationForTest>,
    pub committed: Vec<u64>,
    pub computation_work: Vec<u64>,
    pub committed_facts: Vec<(u64, u64, u64)>,
}
pub(super) fn observe(world: &World, posture: Posture) -> Observed {
    let placement = match posture {
        Posture::Serial => Placement::Serial,
        Posture::Leased { workers } => Placement::Leased(workers),
    };
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _restore = Restore(place(placement));
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
    let mut records = Vec::new();
    let mut computation_work = Vec::new();
    let mut committed = Vec::new();
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
        advancements(); // Discard completed setup/read requests, never mutation events.
        let outcome = request
            .mutate(input)
            .idempotency(&member.key)
            .execute_in_program::<Program>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let observations = advancements();
        let [observed] = observations.as_slice() else {
            panic!("one mutation advancement: {observations:?}")
        };
        assert!(observed.refusal.is_none());
        records.push(observed.clone());
        match outcome {
            Outcome::Committed { result, receipt } => {
                let binding = receipt.idempotency_binding();
                let identity = AppliedIdentity::Mutation {
                    operation: *receipt.installed_operation(),
                    key: *binding.key_identity(),
                    intent: *binding.intent_identity(),
                };
                assert_eq!(
                    observed.applied,
                    [
                        worth_query_host::facade::primary_graph::WorthQueryAdvancementEventForTest {
                            kind: EventKind::Commit,
                            identity: identity.clone()
                        },
                        worth_query_host::facade::primary_graph::WorthQueryAdvancementEventForTest {
                            kind: EventKind::Publication,
                            identity
                        },
                    ],
                    "one actual commit followed by its product publication"
                );
                // Bind the system-issued intent to the declared semantic member key.
                // Membership in this list is decided by an observed commit, not submission.
                committed.push(member.key);
                computation_work.push(result.computation_work);
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
                assert!(
                    observed.applied.is_empty(),
                    "the failed member applies no effect"
                );
                assert!(runs().is_empty(), "failed members have no completed run");
                failure = Some((member.key, denied.partition));
                break;
            }
            other => panic!("unexpected member outcome: {other:?}"),
        }
        let reports = runs();
        use worth_query_host::facade::application_contribution::{
            WorthQueryPartitionedComputationFullCause as Cause,
            WorthQueryPartitionedComputationRunForTest as Run,
        };
        let [(Run::Full(Cause::Unretained), Some(report))] = reports.as_slice() else {
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
    Observed {
        failure,
        kernel_work,
        advancements: records,
        committed,
        computation_work,
        committed_facts,
    }
}

#[cfg(test)]
mod tests;
