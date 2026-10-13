//! An already-elapsed deadline is an executable row, not a missing clock door.
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Admission,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::WorthQueryApplicationOutputDemandDenial as Denial,
    primary_graph::{
        WorthQueryAdvancementObservationForTest as Record, WorthQueryOutputDemandDenialKind as Kind,
    },
};

pub(super) fn assert_deadline<R>(outcome: &Result<R, Denial>, records: &[Record]) {
    let Err(Denial::Demand(cause)) = outcome else {
        panic!("present deadline returns the request's typed interruption")
    };
    assert_eq!(
        cause.kind(),
        Kind::ExecutionRequest(Admission::Interrupted(Interruption::DeadlineExceeded)),
        "present deadline keeps its refusal kind by value"
    );
    assert_eq!(records.len(), 1, "one stopped host advancement is observed");
    assert_eq!(
        (
            records[0].report.charged_work(),
            records[0].report.charged_span()
        ),
        (0, 0),
        "a present deadline dispatches and charges no work"
    );
    assert_eq!(
        records[0].applied.len(),
        0,
        "a present deadline applies no commit or publication"
    );
}
