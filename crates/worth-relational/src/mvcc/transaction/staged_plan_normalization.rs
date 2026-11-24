//! Copy the exact staged intents after their symbol meanings are admitted.
use super::footprint::RelationalTransactionFootprint;
use super::normalized_plan::OwnedNormalizedPlan;
use super::overlay::DetachedRelationalTransactionOverlay;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::symbols::data::ClientKeySymbolPolicy;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::TransactionId;
use worth_execution::ExecutionRequest;

pub(crate) struct StagedPlanNormalization<'a> {
    pub(crate) policy: ClientKeySymbolPolicy,
    pub(crate) symbols: &'a RuntimeSymbolTable,
}

impl StagedPlanNormalization<'_> {
    pub(crate) fn normalized(
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
