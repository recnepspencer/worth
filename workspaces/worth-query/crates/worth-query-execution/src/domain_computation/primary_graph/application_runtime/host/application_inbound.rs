//! The runtime composes `application_inbound`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost;

impl<Schema: ApplicationSchema> ApplicationInboundHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_inbound_host(&self) -> &dyn ApplicationInboundHost<Schema> {
        self
    }
    fn inbound(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_inbound::WorthQueryInboundProcessing
    {
        &self.runtime.inbound
    }
}
