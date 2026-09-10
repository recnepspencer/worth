use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation;

/// Debit the existing attempt; arithmetic overflow cannot produce a smaller
/// allowance. Owners must state their bounded operation's work units explicitly.
pub(crate) fn reserve(
    work: &mut RetainedStoragePreparation,
    visits: Option<usize>,
) -> Result<(), SignalError> {
    let visits = checked(work, visits)?;
    work.reserve_visits(visits)
        .map_err(|_| SignalError::ConditionalEvaluationWorkExhausted {
            maximum_visits: work.maximum_visits(),
        })
}

pub(crate) fn checked(
    work: &RetainedStoragePreparation,
    count: Option<usize>,
) -> Result<usize, SignalError> {
    count.ok_or(SignalError::ConditionalEvaluationWorkExhausted {
        maximum_visits: work.maximum_visits(),
    })
}
