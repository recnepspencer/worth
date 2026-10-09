use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

pub(super) fn assert_program_committed(
    outcome: bank_server::BankProgramMutationExecution<impl std::fmt::Debug>,
    emits: bool,
) {
    let Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, .. }) = outcome else {
        panic!("unexpected program mutation outcome: {outcome:?}");
    };
    if emits {
        assert!(receipt.emitted_effect_count() > 0);
    }
    let work = receipt
        .mutation_work()
        .expect("a fresh program commit retains actual mutation work");
    assert!(work.decision_fact_count() > 0);
    assert!(work.proposed_fact_count() > 0);
    assert!(work.relational_invariant_execution_count() > 0);
    assert!(work.invariant_work_units() > 0);
}
