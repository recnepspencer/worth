//! State custody survives every observer and refunds on the last release.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement;

fn prior_record() -> (WorthQueryApplicationOutputLineage, PriorComputationRecord) {
    let (lineage, identity) = recorded_settlement();
    let (occurrence, generation, slot) = identity.address();
    let cell = Arc::clone(&lineage.by_source[identity.source()][&occurrence][&generation][slot]);
    let state = lineage.retain_computation(
        sealed_run_for_lineage_test(),
        None,
        None,
        lineage.prepay_computation_fork_scan_for_test(),
    );
    cell.get().unwrap().mutable.lock().unwrap().computation = state;
    (lineage, PriorComputationRecord { cell })
}

fn held(prior: &PriorComputationRecord) -> Arc<CustodiedComputation> {
    let row = prior.cell.get().unwrap().mutable.lock().unwrap();
    let RecordedComputation::Retained(handle) = &row.computation else {
        panic!("state retained");
    };
    Arc::clone(handle)
}

#[test]
fn a_shared_state_keeps_one_charge_until_the_last_holder_leaves() {
    let (mut lineage, prior) = prior_record();
    let state = held(&prior);
    let bytes = state.fixture_byte_formula();
    let baseline = 0;
    let shared = prior
        .cell
        .get()
        .unwrap()
        .mutable
        .lock()
        .unwrap()
        .computation
        .shared();
    assert_eq!(lineage.retention.retained_bytes(), baseline + bytes);
    let identity = Arc::clone(&prior.cell.get().unwrap().settlement_identity);
    assert!(prior
        .take_capacity(
            Arc::clone(&state),
            &identity,
            &lineage,
            bytes,
            lineage.prepay_computation_fork_scan_for_test()
        )
        .unwrap()
        .is_none());
    assert_eq!(lineage.retention.retained_bytes(), baseline + bytes);
    lineage.by_source.clear();
    lineage.partition_index = Default::default();
    drop(prior);
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes,
        "an independent prior and alias retain custody"
    );
    drop(shared);
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes,
        "the final prior still owns the same charge"
    );
    drop(state);
    assert_eq!(lineage.retention.retained_bytes(), 0);
}

#[test]
fn an_exclusive_displaced_state_can_transfer_its_ticket_once() {
    let (lineage, prior) = prior_record();
    let state = held(&prior);
    let bytes = state.fixture_byte_formula();
    let identity = Arc::clone(&prior.cell.get().unwrap().settlement_identity);
    let capacity = prior
        .take_capacity(
            state,
            &identity,
            &lineage,
            bytes,
            lineage.prepay_computation_fork_scan_for_test(),
        )
        .unwrap()
        .expect("exclusive exact displaced state");
    assert_eq!(capacity.bytes(), bytes);
    assert!(matches!(
        prior
            .cell
            .get()
            .unwrap()
            .mutable
            .lock()
            .unwrap()
            .computation,
        RecordedComputation::Absent(PriorAbsence::Moved)
    ));
    drop(capacity);
    assert_eq!(lineage.retention.retained_bytes(), 0);
}

#[test]
fn a_shared_successor_reserves_in_full_and_cancellation_preserves_the_prior() {
    let (lineage, prior) = prior_record();
    let state = held(&prior);
    let baseline = state.fixture_byte_formula();
    assert_eq!(lineage.retention.retained_bytes(), baseline);
    let mut successor = sealed_run_for_lineage_test();
    let next_bytes = successor.state.fixture_byte_formula();
    successor.cloned_from = Some(Arc::clone(&state));
    drop(successor); // prepublication cancellation has no effect on any prior
    assert_eq!(lineage.retention.retained_bytes(), baseline);
    let mut successor = sealed_run_for_lineage_test();
    successor.cloned_from = Some(Arc::clone(&state));
    let identity = &prior.cell.get().unwrap().settlement_identity;
    let published = lineage.retain_computation(
        successor,
        Some(&prior),
        Some(identity),
        lineage.prepay_computation_fork_scan_for_test(),
    );
    assert!(matches!(published, RecordedComputation::Retained(_)));
    assert_eq!(lineage.retention.retained_bytes(), baseline + next_bytes);
    assert!(matches!(
        prior
            .cell
            .get()
            .unwrap()
            .mutable
            .lock()
            .unwrap()
            .computation,
        RecordedComputation::Retained(_)
    ));
    drop(published);
    assert_eq!(lineage.retention.retained_bytes(), baseline);
    drop(state);
    drop(prior);
    drop(lineage);
}

#[test]
fn a_refused_incoming_state_does_not_evict_an_existing_holder() {
    let (mut lineage, prior) = prior_record();
    let state = held(&prior);
    let baseline = state.fixture_byte_formula();
    assert_eq!(lineage.retention.retained_bytes(), baseline);
    lineage
        .retention
        .install(baseline, std::num::NonZeroUsize::new(1).unwrap());
    let mut successor = sealed_run_for_lineage_test();
    successor.cloned_from = Some(Arc::clone(&state));
    let published = lineage.retain_computation(
        successor,
        Some(&prior),
        Some(&prior.cell.get().unwrap().settlement_identity),
        lineage.prepay_computation_fork_scan_for_test(),
    );
    assert!(matches!(
        published,
        RecordedComputation::Absent(PriorAbsence::Evicted)
    ));
    assert_eq!(lineage.retention.retained_bytes(), baseline);
    assert!(Arc::ptr_eq(&held(&prior), &state));
}

#[test]
fn refusing_exclusive_ticket_growth_preserves_the_displaced_state() {
    let (mut lineage, prior) = prior_record();
    let state = held(&prior);
    let bytes = state.fixture_byte_formula();
    lineage
        .retention
        .install(bytes, std::num::NonZeroUsize::new(1).unwrap());
    let identity = Arc::clone(&prior.cell.get().unwrap().settlement_identity);
    assert!(prior
        .take_capacity(
            state,
            &identity,
            &lineage,
            bytes + 1,
            lineage.prepay_computation_fork_scan_for_test()
        )
        .is_err());
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    assert!(matches!(
        prior
            .cell
            .get()
            .unwrap()
            .mutable
            .lock()
            .unwrap()
            .computation,
        RecordedComputation::Retained(_)
    ));
}

#[test]
fn a_different_displaced_record_cannot_transfer_the_prior_ticket() {
    let (lineage, prior) = prior_record();
    let (_, foreign) = recorded_settlement();
    let state = held(&prior);
    let bytes = state.fixture_byte_formula();
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    assert!(prior
        .take_capacity(
            state,
            &foreign,
            &lineage,
            bytes,
            lineage.prepay_computation_fork_scan_for_test()
        )
        .unwrap()
        .is_none());
    assert!(matches!(
        prior
            .cell
            .get()
            .unwrap()
            .mutable
            .lock()
            .unwrap()
            .computation,
        RecordedComputation::Retained(_)
    ));
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    drop(prior);
    drop(lineage);
}
