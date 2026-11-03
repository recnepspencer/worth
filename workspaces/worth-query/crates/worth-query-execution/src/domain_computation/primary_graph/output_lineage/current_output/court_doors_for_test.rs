//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn source_facts_for_receipt_for_test(
        &self, runtime_authority: u64, schema: &ApplicationSchemaBindingIdentity, observation: &worth_runtime_world::facade::ProductBranchObservation, receipt: &WorthQueryApplicationCommitReceipt, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
        Self::source_facts_for_receipt(self, runtime_authority, schema, observation, receipt, execution)
    }
}
