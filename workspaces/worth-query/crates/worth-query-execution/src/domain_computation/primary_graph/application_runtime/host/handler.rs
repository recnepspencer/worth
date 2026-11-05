//! The runtime composes `handler`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::handler::host::HandlerHost;

impl<Schema: ApplicationSchema> HandlerHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn handler_host(&self) -> &dyn HandlerHost<Schema> {
        self
    }
    fn mutation_handlers(
        &self,
    ) -> &crate::domain_computation::primary_graph::handler::InstalledMutationHandlerRegistry<Schema>
    {
        &self.mutation_handlers
    }
    fn installed_conditionals(&self) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>{
        &self.installed_conditionals
    }
}
