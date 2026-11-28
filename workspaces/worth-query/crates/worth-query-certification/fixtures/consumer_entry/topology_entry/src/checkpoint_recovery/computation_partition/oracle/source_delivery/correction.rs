//! An entry edit retains the ordinary source-completion owner.
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrection;
use worth_query_host::facade::application_entry::*;

pub(in crate::checkpoint_recovery::computation_partition::oracle) fn source<
    const REUSE: bool,
    const MODE: u8,
>(
    application: &Application<REUSE, TOTALS_WORK, 1, MODE>,
    request: &Request<'_, '_, '_>,
    input: EntryCorrection,
    key: u64,
) -> primary_graph::WorthQueryApplicationCommitReceipt {
    let operation = request
        .mutate(input)
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("source capture: {denial:?}"))
        .retain_operation::<OracleProgram<REUSE, TOTALS_WORK, 1, MODE>, CorrectEntryAction>(
            application,
        )
        .unwrap_or_else(|(_, denial)| panic!("source completion owner: {denial:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(request) else {
        panic!("the capability-admitted source completes");
    };
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("source close: {denial:?}"));
    match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
        WorthQuerySourceCompletion::Recorded { .. } => {
            panic!("a fresh source must retain its original commit receipt")
        }
    }
}
