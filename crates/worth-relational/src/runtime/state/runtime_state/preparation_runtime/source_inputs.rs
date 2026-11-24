//! Lend the exact prepared epoch to Source and invariant computation.
use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::candidate_invariant_inputs::CandidateInvariantInputs;
use crate::validation::candidate_source::applied::SourcePreparationInputs;

impl RelationalPreparationRuntime {
    pub(crate) fn candidate_validation_inputs(&self) -> CandidateInvariantInputs<'_> {
        CandidateInvariantInputs {
            config: &self.config,
            schema_contract_runtime: &self.schema_contract_runtime,
            execution_registry: self.execution_registry.as_ref(),
            home_target: self.home_target.is_some(),
            commit_work_budget: self.commit_work_budget.as_ref(),
            instrumentation: &self.services.instrumentation,
            history: &self.history,
            snapshot_capacity: &self.snapshot_capacity,
        }
    }
}

impl<'a> From<&'a RelationalPreparationRuntime> for SourcePreparationInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            validation: runtime.candidate_validation_inputs(),
            publication: runtime.publication_binding(),
            symbols: &runtime.services.symbols,
        }
    }
}
