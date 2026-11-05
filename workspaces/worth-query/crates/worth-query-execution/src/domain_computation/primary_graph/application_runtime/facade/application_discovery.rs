//! The runtime's consumer methods over `domain_computation::primary_graph::application_discovery`; each only calls its subsystem.
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn discovery(&self) -> crate::domain_computation::primary_graph::application_discovery::WorthQueryApplicationDiscovery<'_, Schema>{
        <Self as crate::domain_computation::primary_graph::application_discovery::ApplicationDiscoveryOps<Schema>>::discovery(self)
    }
}
