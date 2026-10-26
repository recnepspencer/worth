//! Genuine sealed structural inputs force fresh owner calls in both postures.
mod attempt;
mod model;
use super::super::{
    ComputationPrior, SealedComputationRun, WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_contribution::ComputationRetention;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world;
use model::*;
use std::sync::{atomic::Ordering, Arc};
fn body(
    run: &SealedComputationRun,
) -> Arc<crate::domain_computation::primary_graph::invariant_projection::StructuralWalkObservation>
{
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
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for accounted in [true, false] {
        let world = world::install(world::Graph::three_hops(false, vec![]));
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let first = attempt::run(
            &world,
            &installed,
            Some(ComputationPrior::new(
                super::edition(),
                Err(Cause::FirstRun),
                None,
            )),
            accounted,
            Cause::FirstRun,
        );
        assert!(first.state.facts.has_structural_walk());
        let original = body(&first);
        let prior = ComputationPrior::new(super::edition(), Ok(Arc::new(first.state)), None);
        let next = attempt::run(
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
