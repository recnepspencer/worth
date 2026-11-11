//! Copy custody of the actual overlay, with no independent input container.
use super::*;
use crate::mvcc::plan_copy_admission::{intent_bytes, reserve_copy, slots};
use crate::transactions::data::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl DetachedRelationalTransactionOverlay {
    fn input_copy_bytes(&self) -> Option<usize> {
        self.batches
            .iter()
            .try_fold(
                slots::<WorkerIntentBatch>(self.batches.len())?,
                |bytes, batch| {
                    bytes
                        .checked_add(batch.name.len())?
                        .checked_add(batch.partition_key.as_ref().map_or(0, String::len))?
                        .checked_add(intent_bytes(&batch.intents)?)
                },
            )?
            .checked_add(self.normalized_client_keys.copy_bytes()?)
    }

    pub(crate) fn copy_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        request
            .consult()
            .map_err(crate::mvcc::plan_copy_admission::request_denial)?;
        let copied_input_memory = reserve_copy(request, self.input_copy_bytes())?;
        Ok(Self {
            effective_records: self.effective_records.clone(),
            batches: self.batches.clone(),
            index: self.index.clone(),
            normalized_client_keys: self.normalized_client_keys.copy(),
            normalization_generation: self.normalization_generation,
            copied_input_memory,
        })
    }

    pub(crate) fn copy_for_ordinary_preview(&self) -> Self {
        Self {
            effective_records: self.effective_records.clone(),
            batches: self.batches.clone(),
            index: self.index.clone(),
            normalized_client_keys: self.normalized_client_keys.copy(),
            normalization_generation: self.normalization_generation,
            copied_input_memory: None,
        }
    }

    pub(in crate::mvcc::transaction) fn reserve_cache_replacement(
        &mut self,
        keys: &std::collections::BTreeSet<String>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        request
            .consult()
            .map_err(crate::mvcc::plan_copy_admission::request_denial)?;
        let bytes = self
            .input_copy_bytes()
            .and_then(|bytes| {
                bytes.checked_add(self.normalized_client_keys.replacement_bytes(keys)?)
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(crate::mvcc::plan_copy_admission::copy_overflow)?;
        if let Some(memory) = &mut self.copied_input_memory {
            memory.resize(bytes).map_err(|denial| {
                crate::mvcc::plan_copy_admission::request_denial(
                    worth_execution::WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    ),
                )
            })?;
        }
        Ok(())
    }

    pub(in crate::mvcc::transaction) fn refund_replaced_inputs(&mut self) {
        // Shrinking an existing reservation cannot exceed any admission limit.
        // Retain the old bill on an unexpected accounting refusal.
        if let Some(bytes) = self
            .input_copy_bytes()
            .and_then(|bytes| u64::try_from(bytes).ok())
            && let Some(memory) = &mut self.copied_input_memory
            && bytes <= memory.bytes()
        {
            let _ = memory.resize(bytes);
        }
    }
}
