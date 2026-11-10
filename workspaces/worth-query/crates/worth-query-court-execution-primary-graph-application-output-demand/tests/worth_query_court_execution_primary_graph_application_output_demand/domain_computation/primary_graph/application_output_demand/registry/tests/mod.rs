pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key_with_identity;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key_with_query_identity;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::required_interest;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::retain_owned_key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::root;

pub use std::sync::Arc;
pub use std::sync::Condvar;
pub use std::sync::Mutex;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::supersession::supersede_predecessors;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRecord;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandWake;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandNotifications;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::*;
#[test]
fn stale_successor_is_denied_without_mutating_newer_or_unrelated_records() {
    let occurrence = occurrence();
    let newer = key("producer", 8, 3);
    let unrelated = key("producer", 2, 4);
    let mut state = DemandRegistryState::default();
    (*state.records_mut_for_test())
        .insert_for_test(newer.clone(), record(occurrence, DemandState::Admitted, 0));
    (*state.records_mut_for_test()).insert_for_test(
        unrelated.clone(),
        record(occurrence, DemandState::Admitted, 0),
    );

    let denial = supersede_predecessors(&mut state, &key("producer", 7, 3))
        .expect_err("an older revision cannot supersede an admitted newer revision");

    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    assert_eq!((*state.records_for_test()).len_for_test(), 2);
    assert!(matches!(
        *(*state.records_for_test())[&newer].state_for_test(),
        DemandState::Admitted
    ));
    assert!(matches!(
        *(*state.records_for_test())[&unrelated].state_for_test(),
        DemandState::Admitted
    ));
    assert_eq!(
        *(*(*(*state.records_for_test())[&newer].wake_for_test()).generation_for_test())
            .lock()
            .unwrap(),
        0
    );
}
#[test]
fn newer_successor_supersedes_only_its_older_occurrence() {
    let occurrence = occurrence();
    let older = key("producer", 7, 3);
    let unrelated = key("producer", 2, 4);
    let mut state = DemandRegistryState::default();
    (*state.records_mut_for_test())
        .insert_for_test(older.clone(), record(occurrence, DemandState::Scheduled, 1));
    (*state.records_mut_for_test()).insert_for_test(
        unrelated.clone(),
        record(occurrence, DemandState::Admitted, 1),
    );

    supersede_predecessors(&mut state, &key("producer", 8, 3))
        .expect("a newer revision supersedes its predecessor");

    assert!(matches!(
        (*state.records_for_test())[&older].state_for_test(),
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        *(*state.records_for_test())[&unrelated].state_for_test(),
        DemandState::Admitted
    ));
    assert_eq!(
        *(*(*(*state.records_for_test())[&older].wake_for_test()).generation_for_test())
            .lock()
            .unwrap(),
        1
    );
}
#[test]
fn scheduling_failure_is_terminal_and_last_interest_releases_the_record() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("producer", 5, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence, DemandState::Scheduling, 1);
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        "producer stopped before scheduling",
    );

    registry.finish_scheduling_for_test(&demand_interest, None, &mut Err(denial));

    assert!(matches!(
        (*(*registry.state_for_test()).lock().unwrap().records_for_test())[&demand_key].state_for_test(),
        DemandState::Failed(failure)
            if failure.kind() == WorthQueryOutputDemandDenialKind::ProducerUnavailable
    ));
    drop(demand_interest);
    assert!(!(*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
    .contains_key_for_test(&demand_key));
}
#[test]
fn required_failure_is_released_with_its_last_interest() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("required", 5, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __required_interests = 1;
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence, DemandState::Scheduling, 1);
        *__base.required_interests_mut_for_test() = __required_interests;
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        "required producer stopped before scheduling",
    );

    registry.finish_scheduling_for_test(&demand_interest, None, &mut Err(denial));
    drop(demand_interest);

    assert!(
        !(*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_for_test())
        .contains_key_for_test(&demand_key),
        "a failed required attempt is not a permanent owner obligation"
    );
}
#[test]
fn deferred_scheduling_returns_to_admitted_while_no_effect_is_terminal() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let deferred_key = key("deferred", 5, 1);
    let terminal_key = key("no-effect", 5, 2);
    for demand_key in [&deferred_key, &terminal_key] {
        (*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_mut_for_test())
        .insert_for_test(
            demand_key.clone(),
            record(occurrence, DemandState::Scheduling, 1),
        );
    }
    let deferred = interest(
        &registry,
        deferred_key.clone(),
        Arc::clone(
            (*(*registry.state_for_test())
                .lock()
                .unwrap()
                .records_for_test())[&deferred_key]
                .wake_for_test(),
        ),
    );
    let terminal = interest(
        &registry,
        terminal_key.clone(),
        Arc::clone(
            (*(*registry.state_for_test())
                .lock()
                .unwrap()
                .records_for_test())[&terminal_key]
                .wake_for_test(),
        ),
    );
    let no_effect_denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::NoEffect,
        "Signal found no effect",
    );

    registry.finish_scheduling_for_test(
        &deferred,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Deferred),
    );
    registry.finish_scheduling_for_test(
        &terminal,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::NoEffect(no_effect_denial)),
    );

    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(matches!(
        *(*state.records_for_test())[&deferred_key].state_for_test(),
        DemandState::Admitted
    ));
    assert!(matches!(
        (*state.records_for_test())[&terminal_key].state_for_test(),
        DemandState::Failed(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::NoEffect
    ));
}
#[test]
fn denied_executor_releases_the_shared_claim_for_its_peer() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("shared-authorization", 1, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence, DemandState::Scheduled, 2);
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let denied = interest(&registry, demand_key.clone(), Arc::clone(&wake));
    let peer = interest(&registry, demand_key, wake);

    assert!(matches!(
        registry.begin_for_test(&denied),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
    ));
    assert!(matches!(
        registry.begin_for_test(&peer),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Pending
    ));
    registry.relinquish_execution(&denied);
    assert!(matches!(
        registry.begin_for_test(&peer),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
    ));
}

#[path = "awaited_claim/mod.rs"]
pub mod awaited_claim;
#[path = "awaited_publication/mod.rs"]
pub mod awaited_publication;
#[path = "checkpoint_custody/mod.rs"]
pub mod checkpoint_custody;
#[path = "closed_retirement/mod.rs"]
pub mod closed_retirement;
#[path = "completed_custody_stages/mod.rs"]
pub mod completed_custody_stages;
#[path = "consumer_custody/mod.rs"]
pub mod consumer_custody;
#[path = "consumption_guards/mod.rs"]
pub mod consumption_guards;
#[path = "current_successor/mod.rs"]
pub mod current_successor;
#[path = "custody_note/mod.rs"]
pub mod custody_note;
#[path = "custody_reader/mod.rs"]
pub mod custody_reader;
#[path = "delivery_binding/mod.rs"]
pub mod delivery_binding;
#[path = "demand_key/mod.rs"]
pub mod demand_key;
#[path = "drop_cascade/mod.rs"]
pub mod drop_cascade;
#[path = "drop_claim_release/mod.rs"]
pub mod drop_claim_release;
#[path = "drop_cleanup/mod.rs"]
pub mod drop_cleanup;
#[path = "eager_debt/mod.rs"]
pub mod eager_debt;
#[path = "occurrence_retirement/mod.rs"]
pub mod occurrence_retirement;
#[path = "performed_release/mod.rs"]
pub mod performed_release;
#[path = "prerequisite_retry/mod.rs"]
pub mod prerequisite_retry;
#[path = "published_retirement/mod.rs"]
pub mod published_retirement;
#[path = "published_take/mod.rs"]
pub mod published_take;
#[path = "ready_reuse/mod.rs"]
pub mod ready_reuse;
#[path = "recovery_posture/mod.rs"]
pub mod recovery_posture;
#[path = "release_walk/mod.rs"]
pub mod release_walk;
#[path = "replacement_preparation/mod.rs"]
pub mod replacement_preparation;
#[path = "requested_source/mod.rs"]
pub mod requested_source;
#[path = "required_lifecycle/mod.rs"]
pub mod required_lifecycle;
#[path = "required_stop/mod.rs"]
pub mod required_stop;
#[path = "required_work/mod.rs"]
pub mod required_work;
#[path = "reservation_baseline/mod.rs"]
pub mod reservation_baseline;
#[path = "retirement_refusal/mod.rs"]
pub mod retirement_refusal;
#[path = "scheduling_interruption/mod.rs"]
pub mod scheduling_interruption;
#[path = "selected_execution_release/mod.rs"]
pub mod selected_execution_release;
#[path = "semantic_epoch/mod.rs"]
pub mod semantic_epoch;
#[path = "source_custody/mod.rs"]
pub mod source_custody;
#[path = "stopped_head/mod.rs"]
pub mod stopped_head;
#[path = "supersession_refund/mod.rs"]
pub mod supersession_refund;
#[path = "walk_bounds/mod.rs"]
pub mod walk_bounds;
#[path = "wave_admission/mod.rs"]
pub mod wave_admission;
