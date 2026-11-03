//! Genuine sealed structural inputs force fresh owner calls in both postures.

pub use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationFullCause as Cause;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationRunForTest as Run;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::ComputationRetention;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world;
pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::model::*;
pub use std::sync::atomic::Ordering;
pub use std::sync::Arc;
pub fn body(
    run: &SealedComputationRun,
) -> Arc<worth_query_execution::domain_computation::primary_graph::structural_walk::StructuralWalkObservation>{
    let bodies: Vec<_> = run
        .state
        .facts
        .facts()
        .filter_map(|(_, fact, _)| match fact {
            Fact::StructuralWalk { observation } => Some(Arc::clone(observation)),
            _ => None,
        })
        .collect();
    assert_eq!(
        bodies.len(),
        1,
        "the actually sealed owner call retains its one original complete value"
    );
    bodies.into_iter().next().unwrap()
}
#[test]
fn actual_retained_walk_runs_fresh_before_incremental_shortcuts_in_both_postures() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for accounted in [true, false] {
        let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::install(worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::Graph::three_hops(false, vec![]));
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let first = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::attempt::run(
            &world,
            &installed,
            Some(ComputationPrior::new(
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::edition(),
                Err(Cause::FirstRun),
                None,
            )),
            accounted,
            Cause::FirstRun,
        );
        assert!(first.state.facts.has_structural_walk());
        let original = body(&first);
        let prior = ComputationPrior::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::edition(), Ok(Arc::new(first.state)), None);
        let next = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::attempt::run(
            &world,
            &installed,
            Some(prior),
            accounted,
            Cause::StructuralWalkRead,
        );
        assert_eq!(
            installed.owner.gathers.load(Ordering::Relaxed),
            2,
            "the real gather ran again"
        );
        let current = body(&next);
        assert!(
            !Arc::ptr_eq(&original, &current),
            "the current admission issued a fresh body"
        );
        assert_ne!(
            original.source(),
            current.source(),
            "the fresh projection owns its selected native pin"
        );
        assert_eq!(
            original, current,
            "unchanged graph truth excludes source cookie and allocation provenance"
        );
    }
}

#[path = "aggregate/mod.rs"]
pub mod aggregate;
#[path = "aggregate_model/mod.rs"]
pub mod aggregate_model;
#[path = "attempt/mod.rs"]
pub mod attempt;
#[path = "model/mod.rs"]
pub mod model;
