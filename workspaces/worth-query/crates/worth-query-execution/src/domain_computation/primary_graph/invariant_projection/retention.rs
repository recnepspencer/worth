//! Retaining installation-owned projection authority never creates mutation authority.
use super::*;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        WorthQueryApplicationInvariantProjectionAuthority {
            value_bindings: self.value_bindings.clone(),
            graph: self.graph.integration_handle(),
            layout: Arc::clone(&self.graph.layout),
            runtime_authority: self.runtime_authority,
            binding_identity: self.graph.binding_identity().clone(),
            entity_resolution: self.graph.retain_entity_resolution_context(),
            program_activation: self.program_activation.clone(),
            authority_identity: NEXT_INVARIANT_PROJECTION_AUTHORITY
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _schema: PhantomData,
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    /// Retain existing composition-root authority without minting it from a
    /// runtime identity. Each reader still gets a distinct selector namespace.
    pub(in crate::domain_computation::primary_graph) fn retain_for_installation(&self) -> Self {
        Self {
            graph: self.graph.clone(),
            layout: Arc::clone(&self.layout),
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            authority_identity: NEXT_INVARIANT_PROJECTION_AUTHORITY
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            entity_resolution: self.entity_resolution.clone(),
            value_bindings: self.value_bindings.clone(),
            program_activation: self.program_activation.clone(),
            _schema: PhantomData,
        }
    }
}
