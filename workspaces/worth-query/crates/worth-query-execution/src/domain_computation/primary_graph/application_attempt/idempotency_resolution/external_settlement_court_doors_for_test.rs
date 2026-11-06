//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn resolve_guarded_workflow_external_settlement_for_test(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<bool, Denial> {
        Self::resolve_guarded_workflow_external_settlement(self, receipt)
    }
}
