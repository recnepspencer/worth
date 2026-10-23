//! Accepted source completion and correction use the same ordinary owning close.
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::recorded::RecordedEntryCorrection;
use crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrection;
use worth_query_host::facade::application_entry::*;

pub(in crate::checkpoint_recovery::computation_partition::oracle) fn limits(
) -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}

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
    }
}

pub(in crate::checkpoint_recovery::computation_partition::oracle) fn reverse<
    const REUSE: bool,
    const MODE: u8,
>(
    application: &Application<REUSE, TOTALS_WORK, 1, MODE>,
    request: &Request<'_, '_, '_>,
    receipt: &primary_graph::WorthQueryApplicationCommitReceipt,
    key: &str,
) -> primary_graph::WorthQueryApplicationCommitReceipt {
    assert!(
        receipt.has_retained_preimage(),
        "the real source retained its prior field"
    );
    let selected = request.corrections().select(receipt, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<RecordedEntryCorrection<CheckpointSchema>, OracleProgram<REUSE, TOTALS_WORK, 1, MODE>, CorrectEntryAction>(application)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("the inverse retains the ordinary source-completion owner"),
    };
    let WorthQuerySourceOnlyAdvance::Completed(completed) = advance else {
        panic!("the derived inverse completes under fresh capability admission");
    };
    let (_, evidence) = completed
        .close_with_evidence(request)
        .unwrap_or_else(|(_, denial)| panic!("inverse close: {denial:?}"));
    match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    }
}
