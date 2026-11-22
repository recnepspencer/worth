//! Retaining installation-owned projection authority on `WorthQueryPrimaryGraphBootstrap`.

use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::ApplicationInvariantProjectionAuthorityInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryPrimaryGraphBootstrap,
};

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        WorthQueryApplicationInvariantProjectionAuthority::from_installation_parts(
            Arc::clone(&self.installed_operations),
            self.value_bindings.clone(),
            self.graph.integration_handle(),
            Arc::clone(self.graph.layout_ref()),
            self.runtime_authority,
            self.graph.binding_identity().clone(),
            self.graph.retain_entity_resolution_context(),
            self.program_activation.clone(),
        )
    }
}
