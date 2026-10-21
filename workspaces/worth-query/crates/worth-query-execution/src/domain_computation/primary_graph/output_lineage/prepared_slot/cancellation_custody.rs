//! Canceling a prepared publication leaves the captured prior and its ticket intact.
use super::super::{
    registry_fixture::recorded_settlement,
    retained_computation::{PriorComputationRecord, RecordedComputation},
};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::sealed_run_for_lineage_test;

#[test]
fn a_canceled_prepared_publication_cannot_take_its_priors_custody() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let (mut lineage, prior_identity) = recorded_settlement();
    let source = prior_identity.source().clone();
    let (occurrence, generation, old_slot) = prior_identity.address();
    let captured = Arc::clone(&lineage.by_source[&source][&occurrence][&generation][old_slot]);
    let retained = lineage.retain_computation(sealed_run_for_lineage_test());
    let RecordedComputation::Retained(state) = &retained else {
        panic!("prior retained");
    };
    let mut sealed = sealed_run_for_lineage_test();
    sealed.cloned_from = Some(Arc::clone(state));
    captured.get().unwrap().mutable.lock().unwrap().computation = retained;
    let coordinate = ProductCoordinate {
        occurrence,
        generation: generation + 1,
    };

    // A private vacancy grants no publication authority. It is prepaid and
    // exercises the production slot's assignment and cancellation lifecycle.

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

    let output_binding_type = lineage.binding_type(&source.output_binding);
    let owner = Arc::new(Mutex::new(lineage));
    let mut prepared = PreparedOutputLineageSlot {
        owner: Arc::clone(&owner),
        output_binding_type,
        source,
        coordinate,
        partition: None,
        identity: Arc::clone(&identity),
        record_cell,
        partition_cell,

        cancellation: Some(Box::new(CancelledLineageSlot {
            identity,
            partition: None,
            next: None,
        })),
        completed_handler_facts: None,
        completed_decision_reuse: None,
        prepared_input_reuse_key: None,
        native_output_witness: None,
        required_publication: None,
        native_prior_checkpoint: None,

        prior_computation: None,
        filled: false,
        computation: PreparedComputationCustody::Unassigned,
    };
    prepared.retain_computation(
        SealedComputationRetention::Produced(sealed),
        Some(PriorComputationRecord::from_recorded_cell_for_test(
            Arc::clone(&captured),
        )),
    );

    drop(prepared);
    let mut lineage = owner.lock().unwrap();

    assert!(matches!(
        captured.get().unwrap().mutable.lock().unwrap().computation,
        RecordedComputation::Retained(_)
    ));
    lineage.cancelled_slots.take();

    lineage.by_source.clear();
    lineage.partition_index = Default::default();
    drop(captured);
}
