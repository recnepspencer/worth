//! Validate selected lanes and admit payload copying before node drafts exist.
use super::{
    NodeArena, NodeHotData, Preparation, RetainedNodeEditDenial, RetainedStoragePreparationDenial,
};

impl NodeArena {
    pub(super) fn validate_retained_node_selection(
        &self,
        indices: &[usize],
        work: &mut Preparation,
    ) -> Result<(), RetainedNodeEditDenial> {
        if indices.is_empty() {
            return Err(RetainedNodeEditDenial::EmptyNodeSelection);
        }
        // Cover order checks, lane validation, payload cloning, and draft installation
        // loops before any allocation or caller edit. Nested storage visits are
        // separately charged by the payload and persistent-container owners.
        for _ in 0..4 {
            work.reserve_visits(indices.len())
                .map_err(RetainedNodeEditDenial::Accounting)?;
        }
        if indices.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(RetainedNodeEditDenial::UnorderedNodeIndices);
        }
        // Validation, copy admission, and the actual clone each read every
        // selected lane. Admit their tree lookups before entering those loops.
        let lookups = [
            self.hot.lookup_steps(),
            self.warm.lookup_steps(),
            self.cold.lookup_steps(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, steps| sum.checked_add(steps))
        .and_then(|steps| steps.checked_mul(3))
        .and_then(|steps| steps.checked_mul(indices.len()))
        .ok_or(RetainedNodeEditDenial::Accounting(
            RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: work.maximum_visits(),
            },
        ))?;
        work.reserve_visits(lookups)
            .map_err(RetainedNodeEditDenial::Accounting)?;
        for &index in indices {
            self.hot
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.warm
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.cold
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.hot[index]
                .as_ref()
                .ok_or(RetainedNodeEditDenial::MissingHotPayload)?;
            work.reserve_visits(std::mem::size_of::<NodeHotData>() + 32)
                .map_err(RetainedNodeEditDenial::Accounting)?;
            let mut copies = crate::logic::evaluation::EvaluationWork::Conditional(work);
            self.warm[index]
                .admit_clone_work(&mut copies)
                .map_err(map_clone_work_denial)?;
            if let Some(cold) = &self.cold[index] {
                cold.admit_clone_work(&mut copies)
                    .map_err(map_clone_work_denial)?;
            }
        }
        Ok(())
    }
}

/// Carry a clone admission refusal without changing its cause. Conversion to
/// Signal's public error remains at the node edit boundary's central door.
pub(super) fn map_clone_work_denial(
    error: crate::data::error::SignalError,
) -> RetainedNodeEditDenial {
    use crate::data::error::SignalError as Error;
    use RetainedStoragePreparationDenial as Denial;
    let denial = match error {
        Error::ConditionalEvaluationWorkExhausted { maximum_visits } => {
            Denial::WorkExhausted { maximum_visits }
        }
        Error::ExecutionCheckpointStopped(stop) => Denial::ExecutionStopped(stop),
        Error::RetainedStorageChargeOverflow => Denial::ChargeOverflow,
        Error::RetainedStorageChargeUnderflow => Denial::ChargeUnderflow,
        Error::RetainedStorageHistoryUnavailable => Denial::RetainedExtentHistoryUnavailable,
        _ => unreachable!("payload clone admission returns retained work refusals"),
    };
    RetainedNodeEditDenial::Accounting(denial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::error::{SignalCheckpointDenial, SignalError};
    #[test]
    fn clone_admission_preserves_request_checkpoint_causes() {
        for cause in [
            SignalCheckpointDenial::Cancelled,
            SignalCheckpointDenial::DeadlineElapsed,
            SignalCheckpointDenial::WorkCounterOverflow,
            SignalCheckpointDenial::WorkCeiling,
            SignalCheckpointDenial::NestedStopped,
        ] {
            let RetainedNodeEditDenial::Accounting(denial) =
                map_clone_work_denial(SignalError::ExecutionCheckpointStopped(cause))
            else {
                panic!("accounting cause must be carried");
            };
            assert_eq!(
                SignalError::retained_storage_denied(denial),
                SignalError::ExecutionCheckpointStopped(cause)
            );
        }
    }
}
