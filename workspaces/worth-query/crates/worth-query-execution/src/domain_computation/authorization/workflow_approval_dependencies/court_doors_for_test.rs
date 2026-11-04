//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowApprovalDependencies {
    #[doc(hidden)]
    pub fn decode_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        encoded: &str,
    ) -> Result<Self, DecodeDenial> {
        Self::decode(execution, encoded)
    }
}
