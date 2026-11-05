//! Sealed test doors over this module's private members.
use super::*;

pub trait AdmitDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    #[doc(hidden)]
    fn admit_inbound_occurrence_for_test(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
    ) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
        Self::admit_inbound_occurrence(self, handle, envelope)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> AdmitDoorsOps<Schema> for HostRuntime where
    Schema: worth_query_installation::facade::ApplicationSchema, {}
