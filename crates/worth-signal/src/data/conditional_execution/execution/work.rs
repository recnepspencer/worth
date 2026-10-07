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
        .map_err(SignalError::retained_storage_denied)
}

pub(crate) fn checked(
    work: &RetainedStoragePreparation,
    count: Option<usize>,
) -> Result<usize, SignalError> {
    count.ok_or(SignalError::ConditionalEvaluationWorkExhausted {
        maximum_visits: work.maximum_visits(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::error::SignalCheckpointDenial;
    use crate::data::retained_storage::RetainedStoragePreparationDenial;

    #[test]
    fn request_checkpoint_causes_survive_conditional_work_debits() {
        for cause in [
            SignalCheckpointDenial::Cancelled,
            SignalCheckpointDenial::DeadlineElapsed,
            SignalCheckpointDenial::WorkCounterOverflow,
            SignalCheckpointDenial::WorkCeiling,
            SignalCheckpointDenial::NestedStopped,
        ] {
            let mut work = RetainedStoragePreparation::new(1);
            let mut checkpoint = |_| Err(RetainedStoragePreparationDenial::ExecutionStopped(cause));
            let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
            assert_eq!(
                reserve(&mut observed, Some(1)),
                Err(SignalError::ExecutionCheckpointStopped(cause)),
            );
        }
    }
}
