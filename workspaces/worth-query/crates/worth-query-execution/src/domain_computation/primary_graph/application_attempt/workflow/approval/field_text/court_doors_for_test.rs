//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowApprovalFieldText {
    #[doc(hidden)]
    pub fn admit_for_test(
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &WorkflowApprovalMeaning,
    ) -> Result<Self, Denial> {
        Self::admit(runtime, execution, basis, meaning)
    }
}
