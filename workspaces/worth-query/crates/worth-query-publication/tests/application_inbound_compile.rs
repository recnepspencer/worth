//! Public host call shape for installed inbound completion.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundReceipt, WorthQueryInboundVerifierHandle,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_query_publication::facade::application_entry::WorthQueryApplicationInboundOccurrencesExt;

#[allow(dead_code)]
fn receive_from_installed_source<'application, Schema: ApplicationSchema>(
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    source: &'application WorthQueryInboundVerifierHandle,
    envelope: &[u8],
    request: &WorthQueryRequestScope,
) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
    application
        .inbound_occurrences()
        .receive(source, envelope, request)
        .execute()
}
