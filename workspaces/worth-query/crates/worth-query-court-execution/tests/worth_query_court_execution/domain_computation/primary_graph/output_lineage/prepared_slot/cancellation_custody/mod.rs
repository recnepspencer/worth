//! Canceling a prepared publication leaves the captured prior and its ticket intact.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::PriorComputationRecord;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::CancelledLineageSlot;
use std::sync::Mutex;
use std::sync::OnceLock;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::computation::PreparedComputationCustody;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::PreparedOutputLineageSlot;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealed_run_for_lineage_test;

#[test]
fn a_canceled_prepared_publication_cannot_take_its_priors_custody() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let (mut lineage, prior_identity) = recorded_settlement();
    let source = prior_identity.source_for_test().clone();
    let (occurrence, generation, old_slot) = prior_identity.address_for_test();
    let captured =
        Arc::clone(&(*lineage.by_source_for_test())[&source][&occurrence][&generation][old_slot]);
    let retained = lineage.retain_computation_for_test(sealed_run_for_lineage_test());
    let RecordedComputation::Retained(state) = &retained else {
        panic!("prior retained");
    };
    let mut sealed = sealed_run_for_lineage_test();
    (*sealed.cloned_from_mut_for_test()) = Some(Arc::clone(state));
    (*(*captured.get().unwrap().mutable_for_test())
        .lock()
        .unwrap()
        .computation_mut_for_test()) = retained;
    let coordinate = ProductCoordinate::new_for_test(occurrence, generation + 1);

    // A private vacancy grants no publication authority. It is prepaid and
    // exercises the production slot's assignment and cancellation lifecycle.

    let identity = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    let record_cell = Arc::new(OnceLock::new());
    (*lineage.by_source_mut_for_test())
        .entry(source.clone())
        .or_default()
        .entry(occurrence)
        .or_default()
        .insert(
            (*coordinate.generation_for_test()),
            vec![Arc::clone(&record_cell)],
        );
    let partition_cell = (*lineage.partition_index_mut_for_test()).insert_vacancy_for_test(
        source.clone(),
        coordinate,
        None,
    );

    let output_binding_type = lineage.binding_type_for_test(&(*source.output_binding_for_test()));
    let owner = Arc::new(Mutex::new(lineage));
    let mut prepared = PreparedOutputLineageSlot::new_for_test(
        Arc::clone(&owner),
        output_binding_type,
        source,
        coordinate,
        None,
        Arc::clone(&identity),
        record_cell,
        partition_cell,
        Some(Box::new(CancelledLineageSlot::new_for_test(
            identity, None, None,
        ))),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        PreparedComputationCustody::Unassigned,
    );
    prepared.retain_computation_for_test(
        SealedComputationRetention::Produced(sealed),
        Some(PriorComputationRecord::from_recorded_cell_for_test(
            Arc::clone(&captured),
        )),
    );

    drop(prepared);
    let mut lineage = owner.lock().unwrap();

    assert!(matches!(
        (*(*captured.get().unwrap().mutable_for_test())
            .lock()
            .unwrap()
            .computation_for_test()),
        RecordedComputation::Retained(_)
    ));
    (*lineage.cancelled_slots_mut_for_test()).take();

    (*lineage.by_source_mut_for_test()).clear();
    (*lineage.partition_index_mut_for_test()) = Default::default();
    drop(captured);
}
