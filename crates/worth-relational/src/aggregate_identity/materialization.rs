//! Exact late storage demand keeps its original request reservation through admission.
use super::preparation::{PlanDenial, PreparedConsequences};
use crate::branch::SelectedRelationalBranchState;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ChargedBytes, MapKernelFailure};

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
    pub(crate) fn materialize(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        budget: Option<&crate::execution::RequestWorkBudget>,
        selected: &SelectedRelationalBranchState,
        working: &mut WorkingState,
    ) -> Result<(), TransactionCommitError> {
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
                            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
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
