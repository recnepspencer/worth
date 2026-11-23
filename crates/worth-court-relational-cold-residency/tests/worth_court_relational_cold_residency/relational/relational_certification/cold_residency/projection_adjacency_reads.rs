//! Cold and warm selected edges remain genuine, and a zero frontier opens nothing.
use super::{projection_adjacency_doors::Door, ring_source::RingRecipe};
use std::sync::atomic::Ordering;
use worth_relational::facade::identity::KindId;
#[test]
fn cold_and_warm_projection_adjacency_reads_preserve_actual_edges() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = RingRecipe::new(execution, false);
    for door in Door::ALL {
        for historical in [false, true] {
            let cold = recipe.reopen(execution);
            let before = cold.runtime.observe_complexity_counters();
            door.read(&cold, execution, historical).unwrap();
            let after_cold = cold.runtime.observe_complexity_counters();
            let reads = cold.source.reads.load(Ordering::Relaxed);
            assert!(reads > 0);
            door.read(&cold, execution, historical).unwrap();
            let after_warm = cold.runtime.observe_complexity_counters();
            assert_eq!(cold.source.reads.load(Ordering::Relaxed), reads);
            let cold_leases =
                after_cold.adjacency_kind_slices_leased - before.adjacency_kind_slices_leased;
            let warm_leases =
                after_warm.adjacency_kind_slices_leased - after_cold.adjacency_kind_slices_leased;
            assert!(warm_leases > 0);
            assert_eq!(
                cold_leases, warm_leases,
                "{door:?} historical={historical}"
            );
            assert_eq!(
                after_cold.visible_authoritative_relation_records_materialized
                    - before.visible_authoritative_relation_records_materialized,
                after_warm.visible_authoritative_relation_records_materialized
                    - after_cold.visible_authoritative_relation_records_materialized,
            );
        }
    }
}
#[test]
fn an_exhausted_frontier_never_demands_an_unvisited_anchor() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = RingRecipe::new(execution, false);
    for historical in [false, true] {
        let cold = recipe.reopen(execution);
        cold.source.damaged.store(true, Ordering::Relaxed);
        let (_, basis) = cold
            .runtime
            .observe_branch(&cold.runtime.main_branch_identity())
            .unwrap();
        let view = if historical {
            cold.runtime
                .read_truth()
                .try_project_historical_version(super::current_read_basis::main_selected_version(
                    &cold.runtime,
                ))
                .unwrap()
        } else {
            cold.runtime
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap()
        };
        assert!(view
            .bounded_incoming_relations_for_frontier(
                execution,
                &std::collections::BTreeSet::from([cold.entities[0]]),
                KindId(2),
                Some(0)
            )
            .unwrap()
            .is_err());
        assert_eq!(cold.source.reads.load(Ordering::Relaxed), 0);
    }
}
