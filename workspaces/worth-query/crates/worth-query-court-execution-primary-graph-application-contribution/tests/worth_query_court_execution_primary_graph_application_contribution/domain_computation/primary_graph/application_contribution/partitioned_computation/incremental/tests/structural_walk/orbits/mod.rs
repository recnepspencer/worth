//! Orbit comparisons use projected completion and preserve disjoint partitions.
mod attempt;
mod model;
mod property;
use super::{Cause, ComputationPrior, ComputationRetention, Run};
use attempt::Attempt;
use model::{Installed, Owner};
use std::sync::Arc;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits as fixture;
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
#[test]
fn orbit_merge_and_split_regather_only_the_changed_partition_and_match_full_execution() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = fixture::decision::identity::install();
    let installed = Installed::new(Owner::default(), ComputationRetention::ProducerOperation);
    let original = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
    assert_eq!(original.outcome, 6);
    let clean = attempt::run(&world, &installed, prior(original), Run::Incremental);
    assert!(clean.gathered.is_empty());
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let merge = fixture::program::swap_successors(&world, &request, 2, 7);
    assert!(matches!(
        fixture::commit(&world, &request, merge, 110),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let merged = attempt::run(&world, &installed, prior(clean), Run::Incremental);
    assert_eq!(merged.gathered, [0]);
    let full = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
    assert_eq!(merged.outcome, full.outcome);
    assert_eq!(merged.outcome, 8);
    let split = fixture::program::swap_successors(&world, &request, 2, 7);
    assert!(matches!(
        fixture::commit(&world, &request, split, 111),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let split = attempt::run(&world, &installed, prior(merged), Run::Incremental);
    assert_eq!(split.gathered, [0]);
    let full = attempt::run(&world, &installed, first(), Run::Full(Cause::FirstRun));
    assert_eq!(split.outcome, full.outcome);
    assert_eq!(split.outcome, 6);
    let again = attempt::run(&world, &installed, prior(split), Run::Incremental);
    assert!(again.gathered.is_empty());
    assert_eq!(again.outcome, 6);
}
