//! Canceling a prepared publication leaves the captured prior and its ticket intact.
use super::super::{
    registry_fixture::recorded_settlement,
    retained_computation::{PriorComputationRecord, RecordedComputation},
};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::sealed_run_for_lineage_test;

#[test]
fn a_canceled_prepared_publication_cannot_take_its_priors_custody() {
    let (mut lineage, prior_identity) = recorded_settlement();
    let source = prior_identity.source().clone();
    let (occurrence, generation, old_slot) = prior_identity.address();
    let captured = Arc::clone(&lineage.by_source[&source][&occurrence][&generation][old_slot]);
    let retained = lineage.retain_computation(
        sealed_run_for_lineage_test(),
        None,
        None,
        lineage.prepay_computation_fork_scan_for_test(),
    );
    let RecordedComputation::Retained(state) = &retained else {
        panic!("prior retained");
    };
    let bytes = state.fixture_byte_formula();
    let mut sealed = sealed_run_for_lineage_test();
    sealed.cloned_from = Some(Arc::clone(state));
    captured.get().unwrap().mutable.lock().unwrap().computation = retained;
    let coordinate = ProductCoordinate {
        occurrence,
        generation: generation + 1,
    };
    // A private vacancy grants no publication authority. It is prepaid and
    // exercises the production slot's assignment and cancellation lifecycle.
    const VACANCY_BYTES: u64 = 1024 * 1024;
    let capacity = lineage.retention.reserve(VACANCY_BYTES).unwrap();
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let record_cell = Arc::new(OnceLock::new());
    lineage
        .by_source
        .entry(source.clone())
        .or_default()
        .entry(occurrence)
        .or_default()
        .insert(coordinate.generation, vec![Arc::clone(&record_cell)]);
    let partition_cell = lineage
        .partition_index
        .insert_vacancy(source.clone(), coordinate, None);
    let computation_fork_scan_bound = lineage.prepay_computation_fork_scan_for_test();
    let owner = Arc::new(Mutex::new(lineage));
    let mut prepared = PreparedOutputLineageSlot {
        owner: Arc::clone(&owner),
        source,
        coordinate,
        partition: None,
        identity: Arc::clone(&identity),
        record_cell,
        partition_cell,
        retained_capacity: Some(capacity),
        cancellation: Some(Box::new(CancelledLineageSlot {
            identity,
            partition: None,
            retained_capacity: None,
            next: None,
        })),
        completed_handler_facts: None,
        completed_decision_reuse: None,
        prepared_input_reuse_key: None,
        native_output_witness: None,
        actual_resources: None,
        prior_computation: None,
        filled: false,
        computation: PreparedComputationCustody::Unassigned,
        computation_fork_scan_bound,
    };
    prepared.retain_computation(
        SealedComputationRetention::Produced(sealed),
        Some(PriorComputationRecord::from_recorded_cell_for_test(
            Arc::clone(&captured),
        )),
    );
    assert_eq!(
        owner.lock().unwrap().retention.retained_bytes(),
        bytes + VACANCY_BYTES
    );
    drop(prepared);
    let mut lineage = owner.lock().unwrap();
    assert_eq!(
        lineage.retention.retained_bytes(),
        bytes + VACANCY_BYTES,
        "the cancellation cue keeps its prepaid metadata, and the prior remains charged"
    );
    assert!(matches!(
        captured.get().unwrap().mutable.lock().unwrap().computation,
        RecordedComputation::Retained(_)
    ));
    lineage.cancelled_slots.take();
    assert_eq!(lineage.retention.retained_bytes(), bytes);
    lineage.by_source.clear();
    lineage.partition_index = Default::default();
    drop(captured);
    assert_eq!(lineage.retention.retained_bytes(), 0);
}
