//! Exact late storage demand keeps its original request reservation through admission.
use super::preparation::{PlanDenial, PreparedConsequences};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ChargedBytes, MapKernelFailure};
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

struct Materialized {
    admitted: u64,
    copied: u64,
}
impl ChargedBytes for Materialized {
    fn additional_charged_bytes(&self) -> u64 {
        self.admitted
    }
}
impl PreparedConsequences {
    pub fn materialize(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        budget: Option<&crate::contracts::execution::request_work::RequestWorkBudget>,
        selected: &SelectedRelationalBranchState,
        working: &mut WorkingState,
        symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Result<(), TransactionCommitError> {
        selected
            .state()
            .read()
            .demand_regions_head(execution, symbols, self.demands.partitions.iter().copied())
            .map_err(|failure| {
                worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(
                    basis, failure,
                )
            })?;
        let (materialized, retention) = super::funding::run(execution, budget, |work| {
            let mut admitted = 0u64;
            let copied = working.materialize_consequence_records(
                selected,
                &self.demands.partitions,
                &self.demands.entities,
                &self.demands.relations,
                |units, bytes| {
                    let overflow = || {
                        MapKernelFailure::Domain(PlanDenial::Execution(
                            crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                        ))
                    };
                    let bytes = bytes.ok_or_else(overflow)?;
                    admitted = admitted.checked_add(bytes).ok_or_else(overflow)?;
                    work.checkpoint(units)?;
                    work.retain(bytes)
                },
            )?;
            Ok(Materialized { admitted, copied })
        })?;
        debug_assert!(materialized.copied <= materialized.admitted);
        self.admission.storage_retention = retention;
        Ok(())
    }
}
