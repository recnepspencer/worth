//! Host entry for an installed inbound source contract.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundReceipt, WorthQueryInboundVerifierHandle,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

/// Borrowed inbound entry. The runtime has already bound its verifier to the
/// declared operation; envelope bytes cannot select source authority.
pub struct WorthQueryApplicationInboundOccurrences<'application, Schema> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
}

/// One bounded envelope received under the runtime's installed source binding.
pub struct WorthQueryApplicationInboundReceive<'application, 'envelope, 'scope, Schema> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    source: &'application WorthQueryInboundVerifierHandle,
    envelope: &'envelope [u8],
    scope: &'scope WorthQueryRequestScope,
}

/// Opens inbound completion progression on an installed application runtime.
pub trait WorthQueryApplicationInboundOccurrencesExt<Schema>
where
    Schema: ApplicationSchema,
{
    fn inbound_occurrences(&self) -> WorthQueryApplicationInboundOccurrences<'_, Schema>;
}

impl<Schema> WorthQueryApplicationInboundOccurrencesExt<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn inbound_occurrences(&self) -> WorthQueryApplicationInboundOccurrences<'_, Schema> {
        WorthQueryApplicationInboundOccurrences { application: self }
    }
}

impl<'application, Schema> WorthQueryApplicationInboundOccurrences<'application, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn receive<'envelope, 'scope>(
        self,
        source: &'application WorthQueryInboundVerifierHandle,
        envelope: &'envelope [u8],
        scope: &'scope WorthQueryRequestScope,
    ) -> WorthQueryApplicationInboundReceive<'application, 'envelope, 'scope, Schema> {
        WorthQueryApplicationInboundReceive {
            application: self.application,
            source,
            envelope,
            scope,
        }
    }
}

impl<Schema> WorthQueryApplicationInboundReceive<'_, '_, '_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn execute(self) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        self.application
            .receive_inbound_occurrence(self.source, self.envelope, self.scope)
    }
}
