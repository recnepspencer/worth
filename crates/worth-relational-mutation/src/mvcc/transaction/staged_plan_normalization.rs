//! Copy the exact staged intents after their symbol meanings are admitted.
use super::footprint::RelationalTransactionFootprint;
use super::normalized_plan::OwnedNormalizedPlan;
use super::overlay::DetachedRelationalTransactionOverlay;
use crate::contracts::symbols::data::client_key_symbol_policy::ClientKeySymbolPolicy;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::primitives::TransactionId;
use worth_execution::ExecutionRequest;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub struct StagedPlanNormalization<'a> {
    pub policy: ClientKeySymbolPolicy,
    pub symbols: &'a RuntimeSymbolTable,
}

impl StagedPlanNormalization<'_> {
    pub fn normalized(
        &self,
        transaction: TransactionId,
        overlay: &mut DetachedRelationalTransactionOverlay,
        footprint: &mut RelationalTransactionFootprint,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OwnedNormalizedPlan, TransactionCommitError> {
        self.normalize(overlay, footprint, request)?;
        OwnedNormalizedPlan::copy_batches(transaction, overlay.batches(), request)
    }

    fn normalize(
        &self,
        overlay: &mut DetachedRelationalTransactionOverlay,
        footprint: &mut RelationalTransactionFootprint,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        request
            .consult()
            .map_err(super::normalized_plan::request_denial)?;
        let policy = self.policy;
        if !policy.interns_requested_strings() {
            return Ok(());
        }
        let mut result = Ok(());
        self.symbols.normalize_client_keys(|symbols| {
            let (entries, outcome) =
                overlay.normalize_client_keys(footprint, symbols, policy, request);
            result = outcome;
            entries
        });
        result
    }
}
