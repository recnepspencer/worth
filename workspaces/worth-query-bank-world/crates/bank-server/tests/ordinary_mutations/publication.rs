use bank_server::{mutations, BankMutationControls};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use super::{
    fixture::{ordinary_read_world, OWNER},
    key, send_for,
};
use crate::support::request_scope;

#[test]
fn unrelated_graph_population_does_not_widen_commit_publication() {
    let baseline = ordinary_read_world("mutation-publication-baseline", 0);
    let expanded = ordinary_read_world("mutation-publication-expanded", 128);
    let baseline_owner = baseline.authenticate(OWNER);
    let expanded_owner = expanded.authenticate(OWNER);
    let baseline_outcome = baseline
        .world
        .runtime
        .mutate(mutations::send_money(send_for(&baseline)))
        .as_principal(&baseline_owner)
        .controls(BankMutationControls::new(
            request_scope(),
            key("mutation-publication-baseline"),
        ))
        .execute();
    let expanded_outcome = expanded
        .world
        .runtime
        .mutate(mutations::send_money(send_for(&expanded)))
        .as_principal(&expanded_owner)
        .controls(BankMutationControls::new(
            request_scope(),
            key("mutation-publication-expanded"),
        ))
        .execute();
    let Ok(WorthQueryApplicationMutationOutcome::Committed {
        receipt: baseline_receipt,
        ..
    }) = &baseline_outcome
    else {
        panic!("baseline mutation did not commit: {baseline_outcome:?}");
    };
    let Ok(WorthQueryApplicationMutationOutcome::Committed {
        receipt: expanded_receipt,
        ..
    }) = &expanded_outcome
    else {
        panic!("expanded mutation did not commit: {expanded_outcome:?}");
    };

    let baseline_work = baseline_receipt
        .mutation_work()
        .expect("baseline mutation work");
    let expanded_work = expanded_receipt
        .mutation_work()
        .expect("expanded mutation work");
    // Counters and touched-record *count* are invariant to unrelated population.
    // Absolute EntityIds differ across separately provisioned worlds (C2).
    assert_eq!(
        baseline_work.decision_fact_count(),
        expanded_work.decision_fact_count()
    );
    assert_eq!(
        baseline_work.proposed_fact_count(),
        expanded_work.proposed_fact_count()
    );
    assert_eq!(
        baseline_work.invariant_state_fact_count(),
        expanded_work.invariant_state_fact_count()
    );
    assert_eq!(
        baseline_work.invariant_work_units(),
        expanded_work.invariant_work_units()
    );
    assert_eq!(
        baseline_work.relational_invariant_execution_count(),
        expanded_work.relational_invariant_execution_count()
    );
    assert_eq!(
        baseline_work.relational_invariant_result_count(),
        expanded_work.relational_invariant_result_count()
    );
    assert_eq!(
        baseline_work.touched_record_count(),
        expanded_work.touched_record_count()
    );
    assert!(baseline_work.touched_record_count() > 0);
    assert_eq!(
        baseline_receipt.changed_record_count(),
        expanded_receipt.changed_record_count()
    );
    assert_eq!(
        baseline_receipt.emitted_effect_count(),
        expanded_receipt.emitted_effect_count()
    );
}
