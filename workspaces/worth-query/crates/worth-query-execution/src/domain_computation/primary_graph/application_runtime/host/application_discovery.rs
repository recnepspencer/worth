//! The runtime composes `application_discovery`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_discovery::host::ApplicationDiscoveryHost;

impl<Schema: ApplicationSchema> ApplicationDiscoveryHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_discovery_host(&self) -> &dyn ApplicationDiscoveryHost<Schema> {
        self
    }
}
