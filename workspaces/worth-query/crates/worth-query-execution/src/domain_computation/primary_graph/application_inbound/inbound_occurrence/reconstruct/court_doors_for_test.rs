//! Sealed test doors over this module's private members.
use super::*;

pub trait ReconstructDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn verify_completed_inbound_for_cleanup_for_test(
        &self,
        correlations: &[ExternalEffectCorrelationIdentity],
        now_unix_seconds: u64,
    ) -> Result<(), Denial> {
        Self::verify_completed_inbound_for_cleanup(self, correlations, now_unix_seconds)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> ReconstructDoorsOps<Schema> for HostRuntime {}
