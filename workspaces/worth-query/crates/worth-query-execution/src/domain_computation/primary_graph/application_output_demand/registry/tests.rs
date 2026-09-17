use std::sync::{Arc, Condvar, Mutex};

use super::supersession::supersede_predecessors;
use super::{
    DemandRecord, DemandRegistryState, DemandState, DemandWake, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandNotifications, WorthQueryOutputDemandRegistry,
    WorthQueryOutputSchedulingResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

mod recovery_posture;
mod semantic_epoch;
mod source_custody;

mod support;
use support::*;

#[test]
fn stale_successor_is_denied_without_mutating_newer_or_unrelated_records() {
    let occurrence = occurrence();
    let newer = key("producer", 8, 3);
    let unrelated = key("producer", 2, 4);
    let mut state = DemandRegistryState::default();
    state
        .records
        .insert(newer.clone(), record(occurrence, DemandState::Admitted, 0));
    state.records.insert(
        unrelated.clone(),
        record(occurrence, DemandState::Admitted, 0),
    );

    let denial = supersede_predecessors(&mut state, &key("producer", 7, 3))
        .expect_err("an older revision cannot supersede an admitted newer revision");

    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    assert_eq!(state.records.len(), 2);
    assert!(matches!(state.records[&newer].state, DemandState::Admitted));
    assert!(matches!(
        state.records[&unrelated].state,
        DemandState::Admitted
    ));
    assert_eq!(*state.records[&newer].wake.generation.lock().unwrap(), 0);
}

#[test]
fn newer_successor_supersedes_only_its_older_occurrence() {
    let occurrence = occurrence();
    let older = key("producer", 7, 3);
    let unrelated = key("producer", 2, 4);
    let mut state = DemandRegistryState::default();
    state
        .records
        .insert(older.clone(), record(occurrence, DemandState::Scheduled, 1));
    state.records.insert(
        unrelated.clone(),
        record(occurrence, DemandState::Admitted, 1),
    );

    supersede_predecessors(&mut state, &key("producer", 8, 3))
        .expect("a newer revision supersedes its predecessor");

    assert!(matches!(
        &state.records[&older].state,
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        state.records[&unrelated].state,
        DemandState::Admitted
    ));
    assert_eq!(*state.records[&older].wake.generation.lock().unwrap(), 1);
}

#[test]
fn scheduling_failure_is_terminal_and_last_interest_releases_the_record() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("producer", 5, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            wake: Arc::clone(&wake),
            ..record(occurrence, DemandState::Scheduling, 1)
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        "producer stopped before scheduling",
    );

    registry.finish_scheduling(&demand_interest, None, &mut Err(denial));

    assert!(matches!(
        &registry.state.lock().unwrap().records[&demand_key].state,
        DemandState::Failed(failure)
            if failure.kind() == WorthQueryOutputDemandDenialKind::ProducerUnavailable
    ));
    drop(demand_interest);
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(&demand_key));
}

#[test]
fn required_failure_is_released_with_its_last_interest() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("required", 5, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            required: true,
            wake: Arc::clone(&wake),
            ..record(occurrence, DemandState::Scheduling, 1)
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);
    let denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        "required producer stopped before scheduling",
    );

    registry.finish_scheduling(&demand_interest, None, &mut Err(denial));
    drop(demand_interest);

    assert!(
        !registry
            .state
            .lock()
            .unwrap()
            .records
            .contains_key(&demand_key),
        "a failed required attempt is not a permanent owner obligation"
    );
}

#[test]
fn retiring_occurrence_closes_live_work_and_completion_cannot_resurrect_it() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("producer", 5, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            wake: Arc::clone(&wake),
            ..record(occurrence, DemandState::Scheduling, 1)
        },
    );
    let demand_interest = interest(&registry, demand_key.clone(), wake);

    registry.release_product_occurrence(occurrence);
    registry.finish_scheduling(
        &demand_interest,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );

    assert!(matches!(
        &registry.state.lock().unwrap().records[&demand_key].state,
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Closed
    ));
    drop(demand_interest);
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(&demand_key));
}

#[test]
fn deferred_scheduling_returns_to_admitted_while_no_effect_is_terminal() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let deferred_key = key("deferred", 5, 1);
    let terminal_key = key("no-effect", 5, 2);
    for demand_key in [&deferred_key, &terminal_key] {
        registry.state.lock().unwrap().records.insert(
            demand_key.clone(),
            record(occurrence, DemandState::Scheduling, 1),
        );
    }
    let deferred = interest(
        &registry,
        deferred_key.clone(),
        Arc::clone(&registry.state.lock().unwrap().records[&deferred_key].wake),
    );
    let terminal = interest(
        &registry,
        terminal_key.clone(),
        Arc::clone(&registry.state.lock().unwrap().records[&terminal_key].wake),
    );
    let no_effect_denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::NoEffect,
        "Signal found no effect",
    );

    registry.finish_scheduling(
        &deferred,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Deferred),
    );
    registry.finish_scheduling(
        &terminal,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::NoEffect(no_effect_denial)),
    );

    let state = registry.state.lock().unwrap();
    assert!(matches!(
        state.records[&deferred_key].state,
        DemandState::Admitted
    ));
    assert!(matches!(
        &state.records[&terminal_key].state,
        DemandState::Failed(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::NoEffect
    ));
}

#[test]
fn retiring_occurrence_removes_unheld_cached_work_and_marks_preparation_closed() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let scheduled = key("scheduled", 1, 1);
    registry.state.lock().unwrap().records.insert(
        scheduled.clone(),
        record(occurrence, DemandState::Scheduled, 0),
    );
    let preparation = registry.begin_source_preparation(occurrence);

    registry.release_product_occurrence(occurrence);

    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&scheduled));
    assert!(state.source_preparations[&occurrence].retired);
    assert!(state
        .source_custody
        .values()
        .all(|custody| custody.prepared_count() == 0));
    drop(state);
    drop(preparation);
    assert!(registry
        .state
        .lock()
        .unwrap()
        .source_preparations
        .is_empty());
}

#[test]
fn denied_executor_releases_the_shared_claim_for_its_peer() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("shared-authorization", 1, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            wake: Arc::clone(&wake),
            ..record(occurrence, DemandState::Scheduled, 2)
        },
    );
    let denied = interest(&registry, demand_key.clone(), Arc::clone(&wake));
    let peer = interest(&registry, demand_key, wake);

    assert!(matches!(
        registry.begin(&denied),
        super::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
    ));
    assert!(matches!(
        registry.begin(&peer),
        super::WorthQueryOutputDemandAdvanceAdmission::Pending
    ));
    registry.relinquish_execution(&denied);
    assert!(matches!(
        registry.begin(&peer),
        super::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
    ));
}
