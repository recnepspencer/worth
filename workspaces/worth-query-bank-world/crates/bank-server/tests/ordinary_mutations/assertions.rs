use bank_domain::schema::BankSchema;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent;

pub(super) fn assert_program_committed<Input>(
    outcome: bank_server::BankProgramMutationExecution<impl std::fmt::Debug>,
    emits: bool,
) where
    Input: ApplicationMutationIntent<BankSchema>,
{
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
