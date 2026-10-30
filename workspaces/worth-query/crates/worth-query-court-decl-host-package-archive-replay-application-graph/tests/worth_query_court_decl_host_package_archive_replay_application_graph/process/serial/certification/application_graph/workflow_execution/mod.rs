pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_execution::*;

#[test]
fn unsigned_invariant_rejection_publishes_neither_business_write_nor_settlement() {
    const REJECTED_RETENTION: u64 = 21;
    let (application, instance, required) = journey(REJECTED_RETENTION);
    reset_candidate_count(REJECTED_RETENTION);
    assert!(matches!(
        execute(&application, &instance, &required, REJECTED_RETENTION, 91030).unwrap(),
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(denial))
            if denial.kind() == WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    ));
    assert_eq!(candidate_count(REJECTED_RETENTION), 1);
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        SEED_RETENTION
    );
    assert!(matches!(
        advance_instance(&application, instance, 91031).unwrap(),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
}
