//! The runtime composes `application_contribution`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost;

impl<Schema: ApplicationSchema> ApplicationContributionHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_contribution_host(&self) -> &dyn ApplicationContributionHost<Schema> {
        self
    }
    fn output_production(&self) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputProduction<Schema>{
        &self.output_production
    }
}
