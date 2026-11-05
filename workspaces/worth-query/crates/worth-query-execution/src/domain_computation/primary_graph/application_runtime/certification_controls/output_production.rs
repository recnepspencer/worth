//! Handover doors to the output-production state this runtime owns; its test doors live there.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Hands courts the output-demand registry this runtime owns; its test doors live there.
    #[doc(hidden)]
    pub fn output_demands_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry
{
        self.output_production.demands()
    }

    /// Hands courts the output-production state this runtime owns; its test doors live there.
    #[doc(hidden)]
    pub fn output_production_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputProduction<Schema>{
        &self.output_production
    }

    /// Hands courts the output-production state this runtime owns, for mutation.
    #[doc(hidden)]
    pub fn output_production_mut_for_test(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputProduction<Schema>{
        &mut self.output_production
    }
}
