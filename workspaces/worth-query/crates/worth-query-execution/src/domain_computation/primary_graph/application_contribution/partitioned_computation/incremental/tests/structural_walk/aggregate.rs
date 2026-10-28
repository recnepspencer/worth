//! A genuinely sealed aggregate input requires a fresh run in both request postures.
use super::{aggregate_model::{Installed, Owner}, attempt, ComputationPrior, SealedComputationRun, Cause, Fact, ComputationRetention};
use std::sync::{Arc, atomic::Ordering};
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
fn body(run: &SealedComputationRun) -> Arc<AggregateObservation> {
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
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for accounted in [true, false] {
        let world = identity::install();
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let first = attempt::run_aggregate(
            &world,
            &installed,
            Some(ComputationPrior::new(
                super::super::edition(),
                Err(Cause::FirstRun),
                None,
            )),
            accounted,
            Cause::FirstRun,
        );
        assert!(first.state.facts.has_aggregate());
        let original = body(&first);
        let prior = ComputationPrior::new(super::super::edition(), Ok(Arc::new(first.state)), None);
        let next = attempt::run_aggregate(
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
