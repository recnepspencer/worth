use crate::commit_strategies::data::{
    CommitStrategyExecutionRegistration, CommitStrategyRegistration,
};
use crate::config::data::{
    AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy, CrossContextPolicy, DurabilityPolicy,
    DurableLogPolicy, MvccConfig, PublicationConfig, RelationIntegrityScopeBudget,
    StorageLayoutConfig, VisibilityCachePolicy,
};
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::{DurabilityMode, DurableStoreLayout};
use crate::runtime::builder::RelationalRuntimeBuilder;
use crate::schema::data::RelationalSchemaRegistry;
use crate::symbols::data::ClientKeySymbolPolicy;
use crate::validation::data::{CustomInvariantRegistration, InvariantCatalog};

pub struct RuntimeSetup<'a> {
    pub(super) builder: &'a mut RelationalRuntimeBuilder,
}

impl<'a> RuntimeSetup<'a> {
    pub fn runtime_name(&mut self, runtime_name: impl Into<String>) -> &mut Self {
        self.builder.overrides.execution.runtime_name = Some(runtime_name.into());
        self
    }

    pub fn diagnostics(&mut self, diagnostics: RelationalDiagnosticsProfile) -> &mut Self {
        self.builder.overrides.diagnostics.profile = Some(diagnostics);
        self
    }

    pub fn compiled_lane_policy(&mut self, compiled_lane_policy: CompiledLanePolicy) -> &mut Self {
        self.builder.overrides.execution.compiled_lane_policy = Some(compiled_lane_policy);
        self
    }

    pub fn relation_integrity_scope_budget(
        &mut self,
        relation_integrity_scope_budget: RelationIntegrityScopeBudget,
    ) -> &mut Self {
        self.builder
            .overrides
            .execution
            .relation_integrity_scope_budget = Some(relation_integrity_scope_budget);
        self
    }
}

pub struct SchemaSetup<'a> {
    pub(super) builder: &'a mut RelationalRuntimeBuilder,
}

impl<'a> SchemaSetup<'a> {
    pub fn schema_registry(&mut self, schema_registry: RelationalSchemaRegistry) -> &mut Self {
        self.builder.overrides.schema.registry = Some(schema_registry);
        self
    }

    pub fn invariant_catalog(&mut self, invariant_catalog: InvariantCatalog) -> &mut Self {
        self.builder.overrides.schema.invariant_catalog = Some(invariant_catalog);
        self
    }

    pub fn custom_invariant(&mut self, custom_invariant: CustomInvariantRegistration) -> &mut Self {
        self.builder.custom_invariants.push(custom_invariant);
        self
    }
}

pub struct StorageSetup<'a> {
    pub(super) builder: &'a mut RelationalRuntimeBuilder,
}

impl<'a> StorageSetup<'a> {
    pub fn entity_capacity(&mut self, capacity: usize) -> &mut Self {
        self.builder.overrides.storage.initial_entity_capacity = Some(capacity);
        self
    }

    pub fn relation_capacity(&mut self, capacity: usize) -> &mut Self {
        self.builder.overrides.storage.initial_relation_capacity = Some(capacity);
        self
    }

    pub fn mvcc(&mut self, mvcc: MvccConfig) -> &mut Self {
        self.builder.overrides.storage.mvcc = Some(mvcc);
        self
    }

    pub fn storage_layout(&mut self, storage_layout: StorageLayoutConfig) -> &mut Self {
        self.builder.overrides.storage.layout = Some(storage_layout);
        self
    }

    pub fn adjacency_policy(&mut self, adjacency_policy: AdjacencyPolicy) -> &mut Self {
        self.builder.overrides.storage.adjacency_policy = Some(adjacency_policy);
        self
    }

    pub fn cross_context_policy(&mut self, cross_context_policy: CrossContextPolicy) -> &mut Self {
        self.builder.overrides.storage.cross_context_policy = Some(cross_context_policy);
        self
    }

    pub fn cascade_delete_policy(
        &mut self,
        cascade_delete_policy: CascadeDeletePolicy,
    ) -> &mut Self {
        self.builder.overrides.storage.cascade_delete_policy = Some(cascade_delete_policy);
        self
    }

    pub fn visibility_cache_policy(
        &mut self,
        visibility_cache_policy: VisibilityCachePolicy,
    ) -> &mut Self {
        self.builder.overrides.visibility.cache_policy = Some(visibility_cache_policy);
        self
    }

    pub fn client_key_symbol_policy(
        &mut self,
        client_key_symbol_policy: ClientKeySymbolPolicy,
    ) -> &mut Self {
        self.builder.overrides.identity.client_key_symbol_policy = Some(client_key_symbol_policy);
        self
    }

    pub fn publication(&mut self, publication: PublicationConfig) -> &mut Self {
        self.builder.overrides.publication.policy = Some(publication);
        self
    }
}

pub struct DurabilitySetup<'a> {
    pub(super) builder: &'a mut RelationalRuntimeBuilder,
}

impl<'a> DurabilitySetup<'a> {
    pub fn durability_mode(&mut self, durability_mode: DurabilityMode) -> &mut Self {
        self.builder.overrides.durability.mode = Some(durability_mode);
        self
    }

    pub fn durable_log_policy(&mut self, durable_log_policy: DurableLogPolicy) -> &mut Self {
        self.builder.overrides.durability.log = Some(durable_log_policy);
        self
    }

    pub fn durability_policy(&mut self, durability_policy: DurabilityPolicy) -> &mut Self {
        self.builder.overrides.durability.policy = Some(durability_policy);
        self
    }

    pub fn durable_store_layout(&mut self, durable_store_layout: DurableStoreLayout) -> &mut Self {
        self.builder.overrides.durability.store_layout = Some(durable_store_layout);
        self
    }
}

pub struct StrategySetup<'a> {
    pub(super) builder: &'a mut RelationalRuntimeBuilder,
}

impl<'a> StrategySetup<'a> {
    pub fn commit_strategy(&mut self, commit_strategy: CommitStrategyRegistration) -> &mut Self {
        self.builder
            .overrides
            .commit_strategies
            .registrations
            .get_or_insert_with(Vec::new)
            .push(commit_strategy);
        self
    }

    pub fn commit_strategy_executor(
        &mut self,
        commit_strategy_executor: CommitStrategyExecutionRegistration,
    ) -> &mut Self {
        self.builder
            .commit_strategy_executors
            .push(commit_strategy_executor);
        self
    }
}
