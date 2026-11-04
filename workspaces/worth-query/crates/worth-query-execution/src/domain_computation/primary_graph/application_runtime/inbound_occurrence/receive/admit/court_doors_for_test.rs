//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    #[doc(hidden)]
    pub fn admit_inbound_occurrence_for_test(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
    ) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
        Self::admit_inbound_occurrence(self, handle, envelope)
    }
}
