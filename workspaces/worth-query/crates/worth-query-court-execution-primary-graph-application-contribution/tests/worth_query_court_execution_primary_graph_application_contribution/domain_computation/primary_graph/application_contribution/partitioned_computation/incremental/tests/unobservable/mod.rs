//! A retained key the next attempt cannot observe marks the partitions that
//! read it, as a moved fact does, and denies nothing. A state whose keys'
//! summed worst-case observation passes the request work is not compared at
//! all.

use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test;
use worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test;
// a2a9ce5d99 made this the complete host request, not a separate kernel quota.
// Setup through FirstRun measured 23,558 and 9,540 units. Keep ample finite
// host headroom and scale both hostile populations from this same grant.
pub const OBSERVATION_WORK_CEILING: usize = 65_536;
pub fn bound_observation_work() {
    bound_advancement_requests_on_this_thread_for_test(Some(
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(OBSERVATION_WORK_CEILING as u64),
        ),
    ));
}
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as Direction;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey as Key;

#[test]
fn an_unobservable_key_gathers_its_partition_again_and_denies_nothing() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();
    let mut state = first.sealed.unwrap().unwrap().state_into_for_test();
    let key = (*state.facts_for_test())
        .facts_for_test()
        .find(|(_, _, readers)| readers.partitions_for_test().len() == 1)
        .map(|(key, _, _)| key.clone())
        .expect("one gather reads the status");
    let Key::Field {
        entity_id, locator, ..
    } = key.clone()
    else {
        panic!("the status is a field's key: {key:?}");
    };
    // No entity of this name is declared, so the key observes as an error.
    let undeclared = Key::Field {
        entity: "Undeclared".to_owned(),
        entity_id,
        locator,
    };
    (*state.facts_mut_for_test()).rename_key_for_test(&key, undeclared);
    let prior = ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(state)), None);

    let next = attempt(&world, &installed, Some(prior));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert_eq!(next.gathered, [1], "only the partition that read it");
    assert_eq!(next.outcome.unwrap(), outcome);
    assert!(
        next.sealed.unwrap().is_some(),
        "the attempt seals and retains"
    );
}

#[test]
fn more_retained_keys_than_the_request_work_runs_in_full_uncompared() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    bound_observation_work();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();
    let mut state = first.sealed.unwrap().unwrap().state_into_for_test();
    let key = (*state.facts_for_test())
        .facts_for_test()
        .find(|(_, _, readers)| readers.partitions_for_test().len() == 1)
        .map(|(key, _, _)| key.clone())
        .expect("one gather reads the status");
    let Key::Field {
        entity_id, locator, ..
    } = key.clone()
    else {
        panic!("the status is a field's key: {key:?}");
    };
    // One more key than the whole request work, each one a comparison no
    // one is charged for.
    let declared = OBSERVATION_WORK_CEILING;
    (*state.facts_mut_for_test()).copy_keys_for_test(
        &key,
        (0..=declared).map(|copy| Key::Field {
            entity: format!("Undeclared{copy}"),
            entity_id,
            locator: locator.clone(),
        }),
    );
    assert!((*state.facts_for_test()).facts_for_test().count() > OBSERVATION_WORK_CEILING);
    let prior = ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(state)), None);

    fact_verification_on_this_thread_for_test();
    let next = attempt(&world, &installed, Some(prior));
    let observed = fact_verification_on_this_thread_for_test();
    assert_eq!(observed.retained_computation_visits, 0);
    assert_eq!(observed.retained_native_visits, 0);
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::ObservationOverBudget), Some(_))]
    ));
    assert_eq!(next.gathered, [0, 1], "every partition is gathered");
    assert_eq!(
        next.outcome.unwrap(),
        outcome,
        "a full run's total and work"
    );
}

#[test]
fn a_few_adjacency_keys_whose_worst_case_passes_the_request_work_run_in_full() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    bound_observation_work();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();
    let mut state = first.sealed.unwrap().unwrap().state_into_for_test();
    let key = (*state.facts_for_test())
        .facts_for_test()
        .find(|(_, _, readers)| readers.partitions_for_test().len() == 1)
        .map(|(key, _, _)| key.clone())
        .expect("one gather reads the status");
    let Key::Field { entity_id, .. } = key.clone() else {
        panic!("the status is a field's key: {key:?}");
    };
    // Three keys, far fewer than the request work, but each may read a
    // third of it and its anchor's list, so together they pass it.
    let declared = OBSERVATION_WORK_CEILING;
    let third = declared / 3;
    (*state.facts_mut_for_test()).copy_keys_for_test(
        &key,
        (0..3).map(|copy| Key::Adjacency {
            relation: format!("Undeclared{copy}"),
            anchor: entity_id,
            direction: Direction::Outgoing,
            maximum_work_units: third,
        }),
    );
    let prior = ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(state)), None);

    fact_verification_on_this_thread_for_test();
    let next = attempt(&world, &installed, Some(prior));
    let observed = fact_verification_on_this_thread_for_test();
    assert_eq!(observed.retained_computation_visits, 0);
    assert_eq!(observed.retained_native_visits, 0);
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::ObservationOverBudget), Some(_))]
    ));
    assert_eq!(next.gathered, [0, 1], "every partition is gathered");
    assert_eq!(
        next.outcome.unwrap(),
        outcome,
        "a full run's total and work"
    );
}
