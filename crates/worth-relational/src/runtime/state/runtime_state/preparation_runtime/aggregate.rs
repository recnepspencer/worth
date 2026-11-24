//! Project the admitted preparation and schema selection into aggregate inputs.
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::applied_mutation::aggregate_inputs::{
    AggregateDraftAdmission, AggregateDraftInputs,
};
use crate::mvcc::RelationalTransactionValidationInput;
use worth_relational_mutation::internal_port::mvcc::transaction::intent::RelationalTransactionIntentOwnerPort as _;

impl<'a> From<&'a RelationalPreparationRuntime> for AggregateDraftInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            work: runtime.commit_work_budget.as_ref(),
            instrumentation: &runtime.services.instrumentation,
            symbols: &runtime.services.symbols,
            storage: &runtime.config.storage,
        }
    }
}

impl<'a> From<&'a RelationalTransactionValidationInput> for AggregateDraftAdmission<'a> {
    fn from(validation: &'a RelationalTransactionValidationInput) -> Self {
        Self {
            schema: validation.schema_authority(),
            basis: validation.basis(),
            materialization_only: validation.intent().materialization_mode().is_some(),
        }
    }
}
