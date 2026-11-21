//! Retaining installation-owned projection authority never creates mutation authority.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub fn from_installation_parts(
        installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        entity_resolution: WorthQueryInstalledEntityResolutionContext,
        program_activation: Option<
            super::super::program_occurrence::WorthQueryProgramActivationCell,
        >,
    ) -> Self {
        Self {
            installed_operations,
            value_bindings,
            graph,
            layout,
            runtime_authority,
            binding_identity,
            entity_resolution,
            program_activation,
            authority_identity: NEXT_INVARIANT_PROJECTION_AUTHORITY
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _schema: PhantomData,
        }
    }

    /// Retain existing composition-root authority without minting it from a
    /// runtime identity. Each reader still gets a distinct selector namespace.
    pub fn retain_for_installation(&self) -> Self {
        Self {
            installed_operations: Arc::clone(&self.installed_operations),
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
