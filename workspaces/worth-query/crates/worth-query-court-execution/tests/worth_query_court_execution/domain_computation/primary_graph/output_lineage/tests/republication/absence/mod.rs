//! Record absence survives both restoration and performed republication.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;

use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::performed;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealed_run_for_lineage_test;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;

#[test]
fn republication_keeps_a_performed_read_without_inventing_cutoff_proofs() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let consumed = Arc::from([]);
        let generation = stage.restored_generation() - 1;
        let witness = Arc::clone(&stage.performed_witness);
        stage.retain(
            generation,
            |row| {
                let mut row = performed(row, &selection, &consumed, &witness);
                row.completed_decision_reuse = None;
                row.prepared_input_reuse_key = None;
                row
            },
            &facts,
        );
        let restored_witness = Arc::clone(&stage.restored_witness);
        stage
            .republish(generation, &facts, &restored_witness)
            .expect("a performed read does not require input-cutoff proofs")
            .expect("the republication installs");
        let row = stage.restored();
        assert!(row.completed_handler_facts.is_some());
        assert!(row.completed_decision_reuse.is_none());
        assert!(row.prepared_input_reuse_key.is_none());
        assert!(matches!(row.mutable.lock().unwrap().computation,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence::Republished)));
    });
}

#[test]
fn restoring_existing_unverified_facts_drops_state_as_restored() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let sealed = sealed_run_for_lineage_test();
    staged(request_execution, |mut stage| {
        let facts: Arc<[Fact]> = Arc::from([source_entity(1)]);
        let computation = stage.lineage.retain_computation(sealed);
        assert!(matches!(computation, RecordedComputation::Retained(_)));
        let generation = stage.restored_generation() - 1;
        let cell = stage.retain(
            generation,
            |mut row| {
                row.mutable.get_mut().unwrap().computation = computation;
                row
            },
            &facts,
        );
        let record = cell.get().unwrap();
        record.restore(Arc::clone(&facts).into(), None);
        assert!(matches!(
            record.mutable.lock().unwrap().computation,
            RecordedComputation::Absent(PriorAbsence::Restored)
        ));
    });
}

#[test]
fn republication_continues_an_opaque_reader_record() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    republish_untracked(
        request_execution,
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse::default()
            .raw_reader_for_test(),
    );
}

#[test]
fn republication_continues_a_request_context_record() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    republish_untracked(
        request_execution,
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse::default()
            .request_context_for_test(),
    );
}

pub fn republish_untracked(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse,
) {
    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let consumed = Arc::from([]);
        let generation = stage.restored_generation() - 1;
        let witness = Arc::clone(&stage.performed_witness);
        stage.retain(
            generation,
            |row| {
                crate::domain_computation::primary_graph::output_lineage::tests::republication::records::untracked(performed(row, &selection, &consumed, &witness), context)
            },
            &facts,
        );
        let witness = Arc::clone(&stage.restored_witness);
        stage
            .republish(generation, &facts, &witness)
            .unwrap()
            .expect("untracked decision context prevents cutoff, not republication");
        let row = stage.restored();
        assert!(row.completed_handler_facts.is_some());
        assert!(row.completed_decision_reuse.is_none());
        assert!(row.prepared_input_reuse_key.is_none());
        assert!(matches!(
            row.mutable.lock().unwrap().computation,
            RecordedComputation::Absent(PriorAbsence::Republished)
        ));
    });
}
