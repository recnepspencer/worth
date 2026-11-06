//! The document-history conditions a producer's application commit runs under.
//!
//! A producer advancing a document history hands its attempt the borrowed
//! current-control conditions of that advancement. The attempt only carries
//! them from entry to commit; at commit it binds them to its own admitted
//! request, and the provider performs the history cutover under them. The
//! conditions and their cutover belong to `product_operation`; this port is
//! the one thing the attempt needs of them.
use crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

pub(in crate::domain_computation::primary_graph) trait HistoryProducerCommit {
    /// Commits the provider's prepared application under these conditions,
    /// bound to the attempt's admitted `producer_request`.
    fn commit_prepared_application(
        &self,
        provider: &WorthQueryPrimaryGraphProvider,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        producer_request: &AdmittedRequestCurrentness,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    >;
}
