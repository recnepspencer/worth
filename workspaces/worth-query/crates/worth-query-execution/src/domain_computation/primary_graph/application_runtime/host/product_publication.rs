//! The runtime composes `product_publication`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::product_publication::host::ProductPublicationHost;

impl<Schema: ApplicationSchema> ProductPublicationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_publication_host(&self) -> &dyn ProductPublicationHost<Schema> {
        self
    }
    fn unpublished_idempotency_disposition(&self) -> crate::domain_computation::primary_graph::provider::WorthQueryUnpublishedIdempotencyDisposition{
        self.runtime
            .primary_provider
            .unpublished_idempotency_disposition()
    }
}
