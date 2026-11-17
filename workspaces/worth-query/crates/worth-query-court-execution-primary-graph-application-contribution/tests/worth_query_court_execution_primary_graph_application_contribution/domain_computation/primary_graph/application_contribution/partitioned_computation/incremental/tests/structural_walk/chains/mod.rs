//! Structural carriage must isolate changed topology and survive another seal.
mod attempt;
mod model;
mod property;
use super::{Cause, ComputationPrior, ComputationRetention, Run, world};
use attempt::Attempt;
use model::{Installed, Owner};
use std::sync::Arc;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks as fixture;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::edition;

fn first() -> Option<ComputationPrior> {
    Some(ComputationPrior::new_for_test(
        edition(),
        Err(Cause::FirstRun),
        None,
    ))
}
fn prior(attempt: Attempt) -> Option<ComputationPrior> {
    Some(ComputationPrior::new_for_test(
        edition(),
        Ok(Arc::new(attempt.sealed.state_into_for_test())),
        None,
    ))
}
fn graph() -> world::Graph {
    world::Graph {
        nodes: 8,
        edges: vec![(0, 1), (1, 2), (2, 3), (4, 5), (5, 6)],
        active: false,
        blocked: vec![],
    }
}

#[test]
fn editing_adding_or_removing_one_walk_member_regathers_only_its_partition() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for change in 0..3 {
        let mut graph = graph();
        if change == 0 {
            graph.edges.push((7, 3));
        }
        let world = world::install(graph);
        let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
        let before = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
        assert_eq!(before.outcome, 7);
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let program = match change {
            0 => fixture::program::replace_next(&world, &request, 7),
            1 => fixture::program::link(&world, &request, 3, 7),
            _ => fixture::program::replace_next(&world, &request, 2),
        };
        assert!(matches!(
            fixture::commit(&world, &request, program, 90 + change),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let carried = attempt::run(&world, &installed, prior(before), Run::Incremental);
        assert_eq!(
            carried.gathered,
            [0],
            "the disjoint chain is carried for change {change}"
        );
        let full = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
        assert_eq!(
            carried.outcome, full.outcome,
            "fresh full execution is the outcome oracle"
        );
        assert_eq!(carried.outcome, if change == 2 { 6 } else { 8 });
        let again = attempt::run(&world, &installed, prior(carried), Run::Incremental);
        assert!(again.gathered.is_empty());
        assert_eq!(again.outcome, full.outcome);
    }
}

#[test]
fn unchanged_independent_walks_carry_across_consecutive_current_body_seals() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = world::install(graph());
    let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
    let first = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
    let second = attempt::run(&world, &installed, prior(first), Run::Incremental);
    assert!(second.gathered.is_empty());
    assert_eq!(second.outcome, 7);
    let third = attempt::run(&world, &installed, prior(second), Run::Incremental);
    assert!(third.gathered.is_empty());
    assert_eq!(third.outcome, 7);
}
