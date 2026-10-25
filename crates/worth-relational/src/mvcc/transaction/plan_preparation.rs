//! Fallible merge-plan preparation retains unpublished dictionary custody.
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
        let allocation = execution.allocation_policy();
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::PlanPreview,
        )?;
        self.ensure_current_basis(&preparation)
            .map_err(TransactionCommitError::conflict)?;
        self.demand_preparation_regions(
            execution,
            &preparation.services.symbols,
            &preparation.index_definitions,
            &preparation.mirror_residency,
        )?;
        if self.last_merged_plan.is_none() {
            let basis = self.basis.clone();
            let (plan, symbols, overlay, footprint) =
                crate::authority::commit::preparation::probe_symbol_rounds(
                    execution,
                    &preparation,
                    &basis,
                    |mut symbols| {
                        self.with_preparation_overlay(|transaction| {
                            if let Some(prior) = &transaction.pending_symbols {
                                symbols.adopt(prior);
                            }
                            let selected =
                                crate::branch::SelectedRelationalBranchState::from_admitted_basis(
                                    &basis,
                                );
                            transaction.validate_staged_branch_locality(
                                selected.state(),
                                symbols.interner(),
                            )?;
                            transaction.reserve_preparation_client_keys(
                                &mut symbols,
                                preparation.config.identity.client_key_symbol_policy,
                                allocation,
                            )?;
                            let runtime = preparation.with_candidate_symbols(&symbols);
                            let intents =
                                transaction.normalized_intents_for_merge(&runtime, allocation)?;
                            let plan = transaction.build_merged_plan_for_state(
                                &runtime,
                                selected.state(),
                                intents,
                            )?;
                            Ok((
                                plan,
                                symbols,
                                transaction.overlay.clone(),
                                transaction.footprint.clone(),
                            ))
                        })
                    },
                )?;
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
