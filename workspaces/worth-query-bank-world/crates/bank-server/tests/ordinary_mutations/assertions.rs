use bank_domain::schema::BankSchema;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::declaration::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};

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
    let declared =
        <<Input as ApplicationMutationIntent<BankSchema>>::Binding as ApplicationMutationBinding<
            BankSchema,
        >>::CANDIDATES
            .resources()
            .maximum_validator_work();
    let declared =
        u64::try_from(declared).expect("the binding ceiling should fit the work counter");
    assert!(
        work.invariant_work_units() <= declared,
        "observed validator work {} exceeded the binding ceiling {declared}",
        work.invariant_work_units()
    );
}
