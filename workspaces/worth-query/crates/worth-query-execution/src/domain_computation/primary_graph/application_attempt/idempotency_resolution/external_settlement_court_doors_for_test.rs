//! Sealed test doors over this module's private members.
use super::*;

pub trait ExternalSettlementCourtDoorsForTestOps<
    Schema: worth_query_installation::facade::ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[doc(hidden)]
    fn resolve_guarded_workflow_external_settlement_for_test(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<bool, Denial> {
        Self::resolve_guarded_workflow_external_settlement(self, receipt)
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ExternalSettlementCourtDoorsForTestOps<Schema> for HostRuntime {}
