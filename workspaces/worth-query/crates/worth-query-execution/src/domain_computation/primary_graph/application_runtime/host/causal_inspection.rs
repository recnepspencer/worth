//! The runtime composes `causal_inspection`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::causal_inspection::host::CausalInspectionHost;

impl<Schema: ApplicationSchema> CausalInspectionHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn causal_inspection_host(&self) -> &dyn CausalInspectionHost<Schema> {
        self
    }
    fn inspection_materialization_prepared(&self) {
        self.runtime
            .primary_provider
            .inspection_materialization_prepared()
    }
}
