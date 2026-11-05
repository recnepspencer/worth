//! What `product_publication` needs from the runtime that composes it.

/// The application identity plus what `product_publication` reads or calls above itself.
pub trait ProductPublicationHost<Schema>:
    crate::domain_computation::execution_runtime::WorthQueryApplicationIdentityHost<Schema>
{
    /// This host as the trait object `product_publication` is handed.
    fn product_publication_host(&self) -> &dyn ProductPublicationHost<Schema>;
    fn unpublished_idempotency_disposition(&self) -> crate::domain_computation::primary_graph::provider::WorthQueryUnpublishedIdempotencyDisposition;
}
