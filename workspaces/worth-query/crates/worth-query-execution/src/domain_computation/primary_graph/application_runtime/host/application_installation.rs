//! The runtime composes `application_installation`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_installation::host::ApplicationInstallationHost;

impl<Schema: ApplicationSchema> ApplicationInstallationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_installation_host(&self) -> &dyn ApplicationInstallationHost<Schema> {
        self
    }
    fn recovered_outputs(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs{
        &self.output_production.recovered()
    }
}
