mod definition;

pub use definition::RelationalRuntimeDefinition;

#[cfg(test)]
mod tests;

use crate::config::data::{
    AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy, CrossContextPolicy, DurabilityPolicy,
    DurableLogPolicy, MvccConfig, PublicationConfig, RelationalRuntimeProfile, StorageLayoutConfig,
    VisibilityCachePolicy,
};
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::runtime::{RelationalRuntime, RelationalRuntimeConfig, RuntimeExtensions};
use crate::schema::data::RelationalSchemaRegistry;
use crate::symbols::data::ClientKeySymbolPolicy;
use crate::validation::data::{CustomInvariantRegistration, InvariantCatalog};

#[derive(Debug, Clone)]
pub struct RelationalRuntimeBuilder {
    profile: RelationalRuntimeProfile,
    runtime_name: Option<String>,
    diagnostics: Option<RelationalDiagnosticsProfile>,
    schema_registry: Option<RelationalSchemaRegistry>,
    invariant_catalog: Option<InvariantCatalog>,
    entity_capacity: Option<usize>,
    relation_capacity: Option<usize>,
    mvcc: Option<MvccConfig>,
    storage_layout: Option<StorageLayoutConfig>,
    publication: Option<PublicationConfig>,
    client_key_symbol_policy: Option<ClientKeySymbolPolicy>,
    visibility_cache_policy: Option<VisibilityCachePolicy>,
    durable_log_policy: Option<DurableLogPolicy>,
    durability_policy: Option<DurabilityPolicy>,
    adjacency_policy: Option<AdjacencyPolicy>,
    cross_context_policy: Option<CrossContextPolicy>,
    residency_budget: Option<crate::config::data::ResidencyBudget>,
    cascade_delete_policy: Option<CascadeDeletePolicy>,
    compiled_lane_policy: Option<CompiledLanePolicy>,
    custom_invariants: Vec<CustomInvariantRegistration>,
    managed_sources: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
}

impl Default for RelationalRuntimeBuilder {
    fn default() -> Self {
        Self {
            profile: RelationalRuntimeProfile::CertificationCore,
            runtime_name: None,
            diagnostics: None,
            schema_registry: None,
            invariant_catalog: None,
            entity_capacity: None,
            relation_capacity: None,
            mvcc: None,
            storage_layout: None,
            publication: None,
            client_key_symbol_policy: None,
            visibility_cache_policy: None,
            durable_log_policy: None,
            durability_policy: None,
            adjacency_policy: None,
            cross_context_policy: None,
            residency_budget: None,
            cascade_delete_policy: None,
            compiled_lane_policy: None,
            custom_invariants: Vec::new(),
            managed_sources: Vec::new(),
        }
    }
}

impl RelationalRuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn profile(mut self, profile: RelationalRuntimeProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn runtime_name(mut self, runtime_name: impl Into<String>) -> Self {
        self.runtime_name = Some(runtime_name.into());
        self
    }

    pub fn diagnostics(mut self, diagnostics: RelationalDiagnosticsProfile) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    pub fn schema_registry(mut self, schema_registry: RelationalSchemaRegistry) -> Self {
        self.schema_registry = Some(schema_registry);
        self
    }

    pub fn invariant_catalog(mut self, invariant_catalog: InvariantCatalog) -> Self {
        self.invariant_catalog = Some(invariant_catalog);
        self
    }

    pub fn custom_invariant(mut self, custom_invariant: CustomInvariantRegistration) -> Self {
        self.custom_invariants.push(custom_invariant);
        self
    }

    /// Maintain an exact schema-bound Source without registering an invariant.
    /// Preparation retains the original request's controls and publication owner.
    pub fn managed_index_source(
        mut self,
        program: crate::validation::candidate_source::ManagedIndexSourceProgram,
    ) -> Self {
        self.managed_sources.push(program);
        self
    }

    pub fn entity_capacity(mut self, capacity: usize) -> Self {
        self.entity_capacity = Some(capacity);
        self
    }

    pub fn relation_capacity(mut self, capacity: usize) -> Self {
        self.relation_capacity = Some(capacity);
        self
    }

    pub fn mvcc(mut self, mvcc: MvccConfig) -> Self {
        self.mvcc = Some(mvcc);
        self
    }

    pub fn storage_layout(mut self, storage_layout: StorageLayoutConfig) -> Self {
        self.storage_layout = Some(storage_layout);
        self
    }

    pub fn publication(mut self, publication: PublicationConfig) -> Self {
        self.publication = Some(publication);
        self
    }

    pub fn client_key_symbol_policy(
        mut self,
        client_key_symbol_policy: ClientKeySymbolPolicy,
    ) -> Self {
        self.client_key_symbol_policy = Some(client_key_symbol_policy);
        self
    }

    pub fn visibility_cache_policy(
        mut self,
        visibility_cache_policy: VisibilityCachePolicy,
    ) -> Self {
        self.visibility_cache_policy = Some(visibility_cache_policy);
        self
    }

    pub fn durable_log_policy(mut self, durable_log_policy: DurableLogPolicy) -> Self {
        self.durable_log_policy = Some(durable_log_policy);
        self
    }

    pub fn durability_policy(mut self, durability_policy: DurabilityPolicy) -> Self {
        self.durability_policy = Some(durability_policy);
        self
    }

    pub fn adjacency_policy(mut self, adjacency_policy: AdjacencyPolicy) -> Self {
        self.adjacency_policy = Some(adjacency_policy);
        self
    }

    pub fn cross_context_policy(mut self, cross_context_policy: CrossContextPolicy) -> Self {
        self.cross_context_policy = Some(cross_context_policy);
        self
    }

    /// Bound the working set of every cold-backed region this runtime reopens.
    pub fn residency_budget(mut self, budget: crate::config::data::ResidencyBudget) -> Self {
        self.residency_budget = Some(budget);
        self
    }

    pub fn cascade_delete_policy(mut self, cascade_delete_policy: CascadeDeletePolicy) -> Self {
        self.cascade_delete_policy = Some(cascade_delete_policy);
        self
    }

    pub fn compiled_lane_policy(mut self, compiled_lane_policy: CompiledLanePolicy) -> Self {
        self.compiled_lane_policy = Some(compiled_lane_policy);
        self
    }

    /// Build a memory-only runtime: its canonical history is its in-memory
    /// log.
    pub fn build(self) -> Result<RelationalRuntime, super::RelationalRuntimeConstructionDenial> {
        self.build_with(None)
    }

    /// Build a runtime that owns `medium` and publishes every commit to it
    /// before the commit becomes live.
    ///
    /// The medium is moved into this one runtime (D1): the builder never holds
    /// it, so cloning the builder cannot share it, recovery moves it into the
    /// rebuilt runtime, and a forked runtime never inherits it.
    pub fn build_durable(
        self,
        medium: Box<dyn crate::durability::published_medium::RelationalDurableMedium>,
    ) -> Result<RelationalRuntime, super::RelationalRuntimeConstructionDenial> {
        self.build_with(Some(medium))
    }

    fn build_with(
        self,
        medium: Option<Box<dyn crate::durability::published_medium::RelationalDurableMedium>>,
    ) -> Result<RelationalRuntime, super::RelationalRuntimeConstructionDenial> {
        self.resolve().build_with(medium)
    }

    /// Resolve declared configuration without constructing a live runtime.
    pub fn resolve(self) -> RelationalRuntimeDefinition {
        let mut config = RelationalRuntimeConfig::for_profile(self.profile);
        if let Some(runtime_name) = self.runtime_name {
            config.execution.runtime_name = runtime_name;
        }
        if let Some(diagnostics) = self.diagnostics {
            config.diagnostics.profile = diagnostics;
        }
        if let Some(schema_registry) = self.schema_registry {
            config.schema.registry = schema_registry;
        }
        if let Some(invariant_catalog) = self.invariant_catalog {
            config.schema.invariant_catalog = invariant_catalog;
        }
        if let Some(entity_capacity) = self.entity_capacity {
            config.storage.initial_entity_capacity = entity_capacity;
        }
        if let Some(relation_capacity) = self.relation_capacity {
            config.storage.initial_relation_capacity = relation_capacity;
        }
        if let Some(mvcc) = self.mvcc {
            config.storage.retention.backend = mvcc.retention_backend;
            config.storage.retention.reclaim_batch_size = mvcc.reclaim_batch_size;
            config.storage.mvcc = mvcc;
        }
        if let Some(storage_layout) = self.storage_layout {
            config.storage.layout = storage_layout;
        }
        if let Some(publication) = self.publication {
            config.publication.policy = publication;
        }
        if let Some(client_key_symbol_policy) = self.client_key_symbol_policy {
            config.identity.client_key_symbol_policy = client_key_symbol_policy;
        }
        if let Some(visibility_cache_policy) = self.visibility_cache_policy {
            config.visibility.cache_policy = visibility_cache_policy;
        }
        if let Some(durability_policy) = self.durability_policy {
            config.durability.policy = durability_policy;
        }
        if let Some(adjacency_policy) = self.adjacency_policy {
            config.storage.adjacency_policy = adjacency_policy;
        }
        if let Some(cross_context_policy) = self.cross_context_policy {
            config.storage.cross_context_policy = cross_context_policy;
        }
        if let Some(residency_budget) = self.residency_budget {
            config.storage.residency_budget = Some(residency_budget);
        }
        if let Some(cascade_delete_policy) = self.cascade_delete_policy {
            config.storage.cascade_delete_policy = cascade_delete_policy;
        }
        if let Some(compiled_lane_policy) = self.compiled_lane_policy {
            config.execution.compiled_lane_policy = compiled_lane_policy;
        }
        if let Some(durable_log_policy) = self.durable_log_policy {
            config.durability.policy.log = durable_log_policy;
        }
        RelationalRuntimeDefinition::new(
            config,
            RuntimeExtensions::new(self.custom_invariants)
                .with_managed_index_sources(self.managed_sources),
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RelationalRuntimeApi;

impl RelationalRuntimeApi {
    pub fn builder() -> RelationalRuntimeBuilder {
        RelationalRuntimeBuilder::new()
    }
}
