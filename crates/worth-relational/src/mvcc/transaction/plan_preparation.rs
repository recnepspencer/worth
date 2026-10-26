//! Fallible merge-plan preparation retains unpublished dictionary custody.
mod round;

use super::BranchBoundRelationalTransaction;
use crate::transactions::data::{MergedCommitPlan, TransactionCommitError};

impl BranchBoundRelationalTransaction {
    /// Prepare a plan under this request, retaining unpublished symbol meaning
    /// in the transaction. Cold storage refusal is a preparation error; only a
    /// successful commit publishes the plan's newly reserved text identities.
    pub fn prepare_merged_plan(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<&MergedCommitPlan, TransactionCommitError> {
        let preparation = runtime.preparation_runtime_snapshot();
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::PlanPreview,
        )?;
        self.ensure_current_basis(&preparation)
            .map_err(TransactionCommitError::conflict)?;
        self.demand_preparation_inputs(
            execution,
            &preparation.services.symbols,
            &preparation.index_definitions,
        )?;
        if self.last_merged_plan.is_none() {
            let round::PreparedPlanRound {
                plan,
                symbols,
                overlay,
                footprint,
            } = round::prepare(execution, &preparation, self)?;
            self.last_merged_plan = Some(plan);
            self.pending_symbols = Some(symbols);
            self.overlay = overlay;
            self.footprint = footprint;
        }
        Ok(self
            .last_merged_plan
            .as_ref()
            .expect("accepted plan remains in this transaction"))
    }
}
