//! One original recovered batch adopted by its genuinely fresh transaction.
use super::super::{normalized_plan, BranchBoundRelationalTransaction};
use crate::transactions::data::{MutationIntent, TransactionCommitError, WorkerIntentBatch};
use std::fmt::Write;
use worth_execution::ExecutionRequest;

impl BranchBoundRelationalTransaction {
    pub(crate) fn stage_recovered_intents(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        commit: crate::history::data::CommitId,
        intents: &[MutationIntent],
    ) -> Result<(), TransactionCommitError> {
        request.consult().map_err(normalized_plan::request_denial)?;
        if !self.overlay.batches.is_empty() || self.overlay.copied_input_memory.is_some() {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::TransactionStagingCardinalityOverflow,
                ),
            ));
        }
        let digits = commit
            .0
            .checked_ilog10()
            .map_or(1, |digits| digits as usize + 1);
        let label_bytes = "recovery-commit-".len().checked_add(digits);
        let bytes = label_bytes
            .and_then(|bytes| bytes.checked_add(normalized_plan::intent_bytes(intents)?))
            .and_then(|bytes| bytes.checked_add(normalized_plan::slots::<WorkerIntentBatch>(1)?));
        let memory = normalized_plan::reserve_copy(request, bytes)?;
        // Install custody before making any newly retained data. A refusal in
        // push_batch consumes its argument while this original owner stays live.
        self.overlay.copied_input_memory = memory;
        self.overlay.batches = Vec::with_capacity(1);
        let mut name =
            String::with_capacity(label_bytes.ok_or_else(normalized_plan::copy_overflow)?);
        write!(&mut name, "recovery-commit-{}", commit.0)
            .expect("String formatting of a native integer is infallible");
        let mut copied = Vec::with_capacity(intents.len());
        for intent in intents {
            request.consult().map_err(normalized_plan::request_denial)?;
            copied.push(intent.clone());
        }
        self.push_batch(
            WorkerIntentBatch {
                name,
                partition_key: None,
                worker_local_only: true,
                intents: copied,
            },
            request.allocation_policy(),
        )
        .map_err(|denial| {
            crate::execution::allocation_commit_error(TransactionCommitError::conflict(
                denial.into_conflict(),
            ))
        })
    }
}
