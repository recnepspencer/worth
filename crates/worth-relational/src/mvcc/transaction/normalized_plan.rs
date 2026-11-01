//! Independent normalized-plan data and its original request reservation.
use crate::transactions::data::{
    MergedCommitPlan, MutationIntent, TransactionCommitError, TransactionId, WorkerIntentBatch,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(crate) struct OwnedNormalizedPlan {
    plan: MergedCommitPlan,
    _memory: Option<ExecutionMemoryReservation>,
}

impl OwnedNormalizedPlan {
    pub(crate) fn plan(&self) -> &MergedCommitPlan {
        &self.plan
    }

    pub(super) fn intents_mut(&mut self) -> &mut Vec<MutationIntent> {
        &mut self.plan.merged_intents
    }

    pub(crate) fn copy_for_request(
        plan: &MergedCommitPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        request.consult().map_err(request_denial)?;
        let memory = reserve_copy(request, intent_bytes(&plan.merged_intents))?;
        Ok(Self {
            plan: plan.clone(),
            _memory: memory,
        })
    }

    pub(super) fn copy_batches(
        transaction_id: TransactionId,
        batches: &[WorkerIntentBatch],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        request.consult().map_err(request_denial)?;
        let count = batches.iter().try_fold(0usize, |count, batch| {
            count.checked_add(batch.intents.len())
        });
        let bytes = count.and_then(|count| {
            batches
                .iter()
                .flat_map(|batch| &batch.intents)
                .try_fold(slots::<MutationIntent>(count)?, |bytes, intent| {
                    bytes.checked_add(intent.copy_storage_bytes()?)
                })
        });
        let memory = reserve_copy(request, bytes)?;
        let mut intents = Vec::with_capacity(count.ok_or_else(copy_overflow)?);
        for intent in batches.iter().flat_map(|batch| &batch.intents) {
            request.consult().map_err(request_denial)?;
            intents.push(intent.clone());
        }
        Ok(Self {
            plan: MergedCommitPlan {
                transaction_id,
                merged_intents: intents,
            },
            _memory: memory,
        })
    }

    // Ordinary preview rollback snapshots remain explicitly outside controlled
    // copy admission. This does not clone the source owner's reservation.
    pub(super) fn copy_for_ordinary_preview(&self) -> Self {
        Self {
            plan: self.plan.clone(),
            _memory: None,
        }
    }
}

pub(crate) fn intent_bytes(intents: &[MutationIntent]) -> Option<usize> {
    intents
        .iter()
        .try_fold(slots::<MutationIntent>(intents.len())?, |bytes, intent| {
            bytes.checked_add(intent.copy_storage_bytes()?)
        })
}

pub(crate) fn slots<T>(count: usize) -> Option<usize> {
    Some(std::alloc::Layout::array::<T>(count).ok()?.size())
}

pub(crate) fn reserve_copy(
    request: &ExecutionRequest<'_, '_>,
    bytes: Option<usize>,
) -> Result<Option<ExecutionMemoryReservation>, TransactionCommitError> {
    request.consult().map_err(request_denial)?;
    let bytes = bytes
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(copy_overflow)?;
    request
        .reserve_retained_memory(bytes)
        .map_err(request_denial)
}

pub(crate) fn request_denial(cause: worth_execution::WorkCeilingDenial) -> TransactionCommitError {
    TransactionCommitError::execution(cause.into())
}

pub(crate) fn copy_overflow() -> TransactionCommitError {
    TransactionCommitError::execution(
        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow.into(),
    )
}
