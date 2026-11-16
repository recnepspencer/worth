//! Expiry is checked at the actual grouped authored entry before work.
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Admission,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::{
        WorthQueryApplicationRequestMutationDenial as Mutation,
        WorthQueryEditEnvelopeExecutionDenial as Denial,
        WorthQueryPerformedMutationExecutionDenial as Execution,
    },
    primary_graph::WorthQueryAdvancementObservationForTest as Record,
};
pub(super) fn assert_deadline<R>(outcome: &Result<R, Denial>, records: &[Record]) {
    let Err(cause) = outcome else {
        panic!("expired envelope refuses admission")
    };
    assert!(matches!(
        cause.cause(),
        Execution::Mutation(Mutation::ExecutionRequest(Admission::Interrupted(
            Interruption::DeadlineExceeded
        )))
    ));
    let [record] = records else {
        panic!("one stopped envelope advancement: {records:?}")
    };
    assert_eq!(
        (record.report.charged_work(), record.report.charged_span()),
        (0, 0),
        "expired entry dispatches and charges no work"
    );
    assert!(
        record.applied.is_empty(),
        "expired entry applies no commit/publication"
    );
}
