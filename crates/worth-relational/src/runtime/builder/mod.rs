mod setup_sections;

#[cfg(test)]
mod tests;

use crate::commit_strategies::data::{
    CommitStrategyExecutionRegistration, CommitStrategyRegistration,
};
use crate::config::data::{
    AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy, CrossContextPolicy, DurabilityPolicy,
    DurableLogPolicy, MvccConfig, PublicationConfig, RelationIntegrityScopeBudget,
    RelationalConfigOverride, RelationalRuntimeProfile, StorageLayoutConfig, VisibilityCachePolicy,
};
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::{DurabilityMode, DurableStoreLayout};
use crate::runtime::{RelationalRuntime, RelationalRuntimeConfig, RuntimeExtensions};
use crate::schema::data::RelationalSchemaRegistry;
use crate::symbols::data::ClientKeySymbolPolicy;
use crate::validation::data::{CustomInvariantRegistration, InvariantCatalog};

pub use setup_sections::{DurabilitySetup, RuntimeSetup, SchemaSetup, StorageSetup, StrategySetup};

#[derive(Debug, Clone)]
pub struct RelationalRuntimeBuilder {
    profile: RelationalRuntimeProfile,
    overrides: RelationalConfigOverride,
    custom_invariants: Vec<CustomInvariantRegistration>,
    commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    post_commit_consumer: std::sync::Arc<dyn crate::publication::PostCommitConsumer>,
}

impl Default for RelationalRuntimeBuilder {
    fn default() -> Self {
        Self {
            profile: RelationalRuntimeProfile::CertificationCore,
            overrides: RelationalConfigOverride::default(),
            custom_invariants: Vec::new(),
            commit_strategy_executors: Vec::new(),
            post_commit_consumer: crate::publication::production_post_commit_consumer(),
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

    pub fn runtime_setup(mut self, configure: impl FnOnce(&mut RuntimeSetup<'_>)) -> Self {
        let mut setup = RuntimeSetup { builder: &mut self };
        configure(&mut setup);
        self
    }

    pub fn schema_setup(mut self, configure: impl FnOnce(&mut SchemaSetup<'_>)) -> Self {
        let mut setup = SchemaSetup { builder: &mut self };
        configure(&mut setup);
        self
    }

    pub fn storage_setup(mut self, configure: impl FnOnce(&mut StorageSetup<'_>)) -> Self {
        let mut setup = StorageSetup { builder: &mut self };
        configure(&mut setup);
        self
    }

    pub fn durability_setup(mut self, configure: impl FnOnce(&mut DurabilitySetup<'_>)) -> Self {
        let mut setup = DurabilitySetup { builder: &mut self };
        configure(&mut setup);
        self
    }

    pub fn strategy_setup(mut self, configure: impl FnOnce(&mut StrategySetup<'_>)) -> Self {
        let mut setup = StrategySetup { builder: &mut self };
        configure(&mut setup);
        self
    }

    pub fn runtime_name(mut self, runtime_name: impl Into<String>) -> Self {
        self.overrides.execution.runtime_name = Some(runtime_name.into());
        self
    }

    pub fn durability_mode(mut self, durability_mode: DurabilityMode) -> Self {
        self.overrides.durability.mode = Some(durability_mode);
        self
    }

    pub fn diagnostics(mut self, diagnostics: RelationalDiagnosticsProfile) -> Self {
        self.overrides.diagnostics.profile = Some(diagnostics);
        self
    }

    pub fn schema_registry(mut self, schema_registry: RelationalSchemaRegistry) -> Self {
        self.overrides.schema.registry = Some(schema_registry);
        self
    }

    pub fn invariant_catalog(mut self, invariant_catalog: InvariantCatalog) -> Self {
        self.overrides.schema.invariant_catalog = Some(invariant_catalog);
        self
    }

    pub fn custom_invariant(mut self, custom_invariant: CustomInvariantRegistration) -> Self {
        self.custom_invariants.push(custom_invariant);
        self
    }

    pub fn commit_strategy(mut self, commit_strategy: CommitStrategyRegistration) -> Self {
        self.overrides
            .commit_strategies
            .registrations
            .get_or_insert_with(Vec::new)
            .push(commit_strategy);
        self
    }

    pub fn commit_strategy_executor(
        mut self,
        commit_strategy_executor: CommitStrategyExecutionRegistration,
    ) -> Self {
        self.commit_strategy_executors
            .push(commit_strategy_executor);
        self
    }

    pub fn post_commit_consumer(
        mut self,
        post_commit_consumer: std::sync::Arc<dyn crate::publication::PostCommitConsumer>,
    ) -> Self {
        self.post_commit_consumer = post_commit_consumer;
        self
    }

    pub fn entity_capacity(mut self, capacity: usize) -> Self {
        self.overrides.storage.initial_entity_capacity = Some(capacity);
        self
    }

    pub fn relation_capacity(mut self, capacity: usize) -> Self {
        self.overrides.storage.initial_relation_capacity = Some(capacity);
        self
    }

    pub fn mvcc(mut self, mvcc: MvccConfig) -> Self {
        self.overrides.storage.mvcc = Some(mvcc);
        self
    }

    pub fn storage_layout(mut self, storage_layout: StorageLayoutConfig) -> Self {
        self.overrides.storage.layout = Some(storage_layout);
        self
    }

    pub fn publication(mut self, publication: PublicationConfig) -> Self {
        self.overrides.publication.policy = Some(publication);
        self
    }

    pub fn client_key_symbol_policy(
        mut self,
        client_key_symbol_policy: ClientKeySymbolPolicy,
    ) -> Self {
        self.overrides.identity.client_key_symbol_policy = Some(client_key_symbol_policy);
        self
    }

    pub fn visibility_cache_policy(
        mut self,
        visibility_cache_policy: VisibilityCachePolicy,
    ) -> Self {
        self.overrides.visibility.cache_policy = Some(visibility_cache_policy);
        self
    }

    pub fn durable_log_policy(mut self, durable_log_policy: DurableLogPolicy) -> Self {
        self.overrides.durability.log = Some(durable_log_policy);
        self
    }

    pub fn durability_policy(mut self, durability_policy: DurabilityPolicy) -> Self {
        self.overrides.durability.policy = Some(durability_policy);
        self
    }

    pub fn durable_store_layout(mut self, durable_store_layout: DurableStoreLayout) -> Self {
        self.overrides.durability.store_layout = Some(durable_store_layout);
        self
    }

    pub fn adjacency_policy(mut self, adjacency_policy: AdjacencyPolicy) -> Self {
        self.overrides.storage.adjacency_policy = Some(adjacency_policy);
        self
    }

    pub fn cross_context_policy(mut self, cross_context_policy: CrossContextPolicy) -> Self {
        self.overrides.storage.cross_context_policy = Some(cross_context_policy);
        self
    }

    pub fn cascade_delete_policy(mut self, cascade_delete_policy: CascadeDeletePolicy) -> Self {
        self.overrides.storage.cascade_delete_policy = Some(cascade_delete_policy);
        self
    }

    pub fn compiled_lane_policy(mut self, compiled_lane_policy: CompiledLanePolicy) -> Self {
        self.overrides.execution.compiled_lane_policy = Some(compiled_lane_policy);
        self
    }

    pub fn relation_integrity_scope_budget(
        mut self,
        relation_integrity_scope_budget: RelationIntegrityScopeBudget,
    ) -> Self {
        self.overrides.execution.relation_integrity_scope_budget =
            Some(relation_integrity_scope_budget);
        self
    }

    pub fn build(self) -> RelationalRuntime {
        RelationalRuntime::build_from_extensions(
            RelationalRuntimeConfig::resolved(self.profile, self.overrides),
            RuntimeExtensions::new(self.custom_invariants, self.commit_strategy_executors)
                .with_post_commit_consumer(self.post_commit_consumer),
        )
    }
}
