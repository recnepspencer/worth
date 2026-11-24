//! Bind one admitted root preparation round to its mutation owners.
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::applied_mutation::inputs::AppliedMutationInputs;

impl<'a> From<&'a RelationalPreparationRuntime> for AppliedMutationInputs<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            storage: &runtime.config.storage,
            key_policy: runtime.config.identity.client_key_symbol_policy,
            work: &runtime.commit_work_budget,
            record_identity: &runtime.record_identity,
            authored_coupling: runtime.authored_coupling.as_ref(),
        }
    }
}

impl<'a> From<&'a mut RelationalPreparationRuntime> for AppliedMutationInputs<'a> {
    fn from(runtime: &'a mut RelationalPreparationRuntime) -> Self {
        (&*runtime).into()
    }
}
