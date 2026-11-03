//! A genuinely sealed aggregate input requires a fresh run in both request postures.
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::aggregate_model::Installed;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::aggregate_model::Owner;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::attempt;
use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior;
use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationFullCause as Cause;
use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::ComputationRetention;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use worth_query_execution::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
pub fn body(run: &SealedComputationRun) -> Arc<AggregateObservation> {
    let values: Vec<_> = run
        .state
        .facts
        .facts()
        .filter_map(|(_, fact, _)| match fact {
            Fact::Aggregate { observation } => Some(observation.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 1);
    values.into_iter().next().unwrap()
}
#[test]
fn actual_aggregate_inputs_force_fresh_computation_before_reuse_in_both_postures() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for accounted in [true, false] {
        let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let first = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::attempt::run_aggregate(
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
        assert!(first.state.facts.has_aggregate());
        let original = body(&first);
        let prior = ComputationPrior::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::edition(), Ok(Arc::new(first.state)), None);
        let next = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::structural_walk::attempt::run_aggregate(
            &world,
            &installed,
            Some(prior),
            accounted,
            Cause::AggregateRead,
        );
        assert_eq!(installed.owner.gathers.load(Ordering::Relaxed), 2);
        let current = body(&next);
        assert!(!Arc::ptr_eq(&original, &current));
        assert_ne!(original.source(), current.source());
        assert_eq!(
            original, current,
            "actual truth excludes source cookie, payer and incident history"
        );
    }
}
