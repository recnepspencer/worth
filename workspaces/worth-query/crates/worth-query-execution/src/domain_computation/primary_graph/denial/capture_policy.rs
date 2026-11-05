//! A refused checkpoint capture policy is an installation execution denial.
use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

impl From<crate::domain_computation::request_execution::advancement::WorthQueryCapturePolicyRefusal>
    for WorthQueryPrimaryGraphInstallationDenial
{
    fn from(
        refusal: crate::domain_computation::request_execution::advancement::WorthQueryCapturePolicyRefusal,
    ) -> Self {
        use crate::domain_computation::request_execution::advancement::WorthQueryCapturePolicyRefusal as Refusal;
        use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind;
        let (kind, subject) = match refusal {
            Refusal::UnleasedFunding => (
                Kind::ResourceEnvelopeMismatch,
                "original-request checkpoint capture requires leased backing; serial system allocation is unsupported",
            ),
            Refusal::ForeignCaptureLease => (
                Kind::ExecutionResource {
                    denial: crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::NestedLeaseMisuse,
                    partition_identity: None,
                    policy_ancestor: None,
                },
                "checkpoint capture requires a lease descending from the transition's funding lease",
            ),
            Refusal::UnleasedCapture => (
                Kind::ResourceEnvelopeMismatch,
                "checkpoint capture requires leased backing; system allocation is unsupported",
            ),
        };
        Self::new(
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind },
            subject,
        )
    }
}
