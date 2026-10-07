//! Record absence survives both restoration and performed republication.

use super::records::performed;
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::{sealed_run_for_lineage_test, PriorAbsence},
    application_query::WorthQueryObservedSourceSelection,
    output_lineage::retained_computation::RecordedComputation,
};

#[test]
fn republication_keeps_a_performed_read_without_inventing_cutoff_proofs() {
    staged(|mut stage| {
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
            .republish(generation, &facts, &restored_witness, &mut admission(4_096))
            .expect("a performed read does not require input-cutoff proofs")
            .expect("the republication installs");
        let row = stage.restored();
        assert!(row.completed_handler_facts.is_some());
        assert!(row.completed_decision_reuse.is_none());
        assert!(row.prepared_input_reuse_key.is_none());
        assert!(matches!(row.mutable.lock().unwrap().computation,
            crate::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(
                crate::domain_computation::primary_graph::application_contribution::PriorAbsence::Republished)));
    });
}

#[test]
fn restoring_existing_unverified_facts_drops_state_as_restored() {
    staged(|mut stage| {
        let facts: Arc<[Fact]> = Arc::from([source_entity(1)]);
        let computation =
            stage
                .lineage
                .retain_computation(sealed_run_for_lineage_test(), None, None);
        assert!(matches!(computation, RecordedComputation::Retained { .. }));
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
        record.restore(Arc::clone(&facts), None, None);
        assert!(matches!(
            record.mutable.lock().unwrap().computation,
            RecordedComputation::Absent(PriorAbsence::Restored)
        ));
    });
}

#[test]
fn republication_continues_an_opaque_reader_record() {
    republish_untracked(
        crate::domain_computation::primary_graph::DecisionContextUse::default()
            .raw_reader_for_test(),
    );
}

#[test]
fn republication_continues_a_request_context_record() {
    republish_untracked(
        crate::domain_computation::primary_graph::DecisionContextUse::default()
            .request_context_for_test(),
    );
}

fn republish_untracked(context: crate::domain_computation::primary_graph::DecisionContextUse) {
    staged(|mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let consumed = Arc::from([]);
        let generation = stage.restored_generation() - 1;
        let witness = Arc::clone(&stage.performed_witness);
        stage.retain(
            generation,
            |row| {
                super::records::untracked(performed(row, &selection, &consumed, &witness), context)
            },
            &facts,
        );
        let witness = Arc::clone(&stage.restored_witness);
        stage
            .republish(generation, &facts, &witness, &mut admission(4_096))
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
