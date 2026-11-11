//! Read-only managed Source storage observation for certification consumers.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_query::managed_index::certification::{
    WorthQueryCertificationManagedIndexPages, WorthQueryCertificationManagedIndexPagesDenial,
};
use worth_query_installation::facade::ApplicationSchema;

pub trait WorthQueryCertificationManagedIndexRuntimeExt {
    /// Scans all currently registered Native branch images and their retained Source scopes.
    /// Counts shared provider node pages once per Source program across those images.
    /// Explicit diagnostic work; it grants no read authority and excludes historical-only pins.
    fn observe_certification_managed_index_pages(
        &self,
    ) -> Result<
        Vec<WorthQueryCertificationManagedIndexPages>,
        WorthQueryCertificationManagedIndexPagesDenial,
    >;
}
impl<S: ApplicationSchema> WorthQueryCertificationManagedIndexRuntimeExt
    for WorthQueryPrimaryGraphApplicationRuntime<S>
{
    fn observe_certification_managed_index_pages(
        &self,
    ) -> Result<
        Vec<WorthQueryCertificationManagedIndexPages>,
        WorthQueryCertificationManagedIndexPagesDenial,
    > {
        self.runtime
            .primary_provider
            .graph
            .managed_derived_views()
            .inspect_source_pages()
    }
}
