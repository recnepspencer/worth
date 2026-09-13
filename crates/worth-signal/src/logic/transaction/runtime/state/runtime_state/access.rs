use crate::data::graph::{EvaluationStrategy, SignalGraph};

use crate::data::telemetry::RuntimeTelemetry;

use crate::logic::checkpoint::CheckpointRuntime;

use crate::logic::events::EventBus;

use crate::schema::data::SignalSchemaRegistry;

use super::super::super::config::SignalRuntimeConfig;

use super::super::merge::{
    FrozenAspectMergePolicyRegistry, FrozenConflictIsolationRegistry, FrozenConflictPolicyRegistry,
    FrozenDeletionPolicyRegistry, FrozenIdentityMatcherRegistry, FrozenMergeBaseStrategyRegistry,
    FrozenMergeStrategyRegistry, FrozenSourceOnlyPolicyRegistry,
};

use super::super::observer::RuntimeObserver;

use super::super::runtime_observation::RuntimeObservationRegistry;

use super::{SignalGraphMut, SignalRuntime};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn config(&self) -> &SignalRuntimeConfig<T> {
        self.assert_construction_graph_access();
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SignalRuntimeConfig<T> {
        self.assert_construction_graph_access();
        self.config.sync_graph_capacity(&self.graph);
        &mut self.config
    }

    pub fn graph(&self) -> &SignalGraph {
        self.assert_construction_graph_access();
        &self.graph
    }

    pub fn schema_registry(&self) -> &SignalSchemaRegistry {
        &self.schema_registry
    }

    pub fn merge_strategy_registry(&self) -> &FrozenMergeStrategyRegistry {
        &self.merge_strategy_registry
    }

    pub fn merge_base_strategy_registry(&self) -> &FrozenMergeBaseStrategyRegistry {
        &self.merge_base_strategy_registry
    }

    pub fn aspect_merge_policy_registry(&self) -> &FrozenAspectMergePolicyRegistry {
        &self.aspect_merge_policy_registry
    }

    pub fn conflict_policy_registry(&self) -> &FrozenConflictPolicyRegistry {
        &self.conflict_policy_registry
    }

    pub fn conflict_isolation_registry(&self) -> &FrozenConflictIsolationRegistry {
        &self.conflict_isolation_registry
    }

    pub fn identity_matcher_registry(&self) -> &FrozenIdentityMatcherRegistry {
        &self.identity_matcher_registry
    }

    pub fn source_only_policy_registry(&self) -> &FrozenSourceOnlyPolicyRegistry {
        &self.source_only_policy_registry
    }

    pub fn deletion_policy_registry(&self) -> &FrozenDeletionPolicyRegistry {
        &self.deletion_policy_registry
    }

    pub fn validate_schema_bindings(&self) -> Result<(), crate::data::error::SignalError> {
        self.assert_construction_graph_access();
        self.graph
            .validate_schema_bindings_against(&self.schema_registry)
    }

    pub fn validate_merge_semantics(&self) -> Result<(), crate::data::error::SignalError> {
        self.assert_construction_graph_access();
        self.graph.validate_merge_semantics_against(
            &self.schema_registry,
            &self.merge_strategy_registry,
            &self.aspect_merge_policy_registry,
            &self.conflict_isolation_registry,
            &self.conflict_policy_registry,
            &self.identity_matcher_registry,
            &self.source_only_policy_registry,
            &self.deletion_policy_registry,
        )
    }

    pub fn observe(&self) -> RuntimeObserver<'_, D, I, E, Ctx, T> {
        RuntimeObserver::new(self)
    }

    pub fn derive_evaluation_strategy(&self) -> EvaluationStrategy {
        self.assert_construction_graph_access();
        self.graph.derive_evaluation_strategy()
    }

    pub fn graph_mut(&mut self) -> SignalGraphMut<'_, D, I, E, Ctx, T> {
        self.assert_construction_graph_access();
        self.config.sync_graph_capacity(&self.graph);
        SignalGraphMut { runtime: self }
    }

    pub fn clear_live_branch_mutation_residue(&mut self) {
        self.assert_construction_graph_access();
        self.graph.clear_branch_mutation_nodes();
    }

    pub(in crate::logic::transaction::runtime::state) fn assert_construction_graph_access(&self) {
        self.assert_construction_state_access();
    }

    pub(in crate::logic::transaction::runtime::state) fn assert_construction_state_access(&self) {
        assert!(
            !self.owner_services.is_sealed(),
            "legacy runtime state access is unavailable after owner-service sealing"
        );
    }

    pub fn checkpoint(&self) -> &CheckpointRuntime<D, I> {
        self.assert_construction_state_access();
        &self.checkpoint
    }

    pub fn event_bus(&self) -> &EventBus<E, D, Ctx> {
        &self.event_bus
    }

    pub fn event_bus_mut(&mut self) -> &mut EventBus<E, D, Ctx> {
        assert!(
            !self.owner_services.is_sealed(),
            "event subscribers cannot be configured after owner-service sealing"
        );
        self.event_bus
            .set_telemetry_capture(self.graph.captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            ));
        &mut self.event_bus
    }

    pub fn observations(&self) -> &RuntimeObservationRegistry<D, I, E, Ctx, T> {
        &self.observations
    }

    pub fn observations_mut(&mut self) -> &mut RuntimeObservationRegistry<D, I, E, Ctx, T> {
        assert!(
            !self.owner_services.is_sealed(),
            "observation listeners cannot be configured after owner-service sealing"
        );
        &mut self.observations
    }

    pub fn telemetry(&self) -> &RuntimeTelemetry {
        self.assert_construction_state_access();
        &self.telemetry
    }

    pub(in crate::logic::transaction::runtime) fn with_telemetry(
        &mut self,
        update: impl FnOnce(&mut RuntimeTelemetry),
    ) {
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            update(&mut self.telemetry);
        }
    }

    pub(in crate::logic::transaction::runtime) fn with_resource_telemetry(
        &mut self,
        update: impl FnOnce(&mut crate::data::telemetry::ResourceTelemetry),
    ) {
        self.with_telemetry(|telemetry| update(&mut telemetry.resource));
    }

    pub(in crate::logic::transaction::runtime) fn telemetry_snapshot(&self) -> RuntimeTelemetry {
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            self.telemetry
        } else {
            RuntimeTelemetry::default()
        }
    }
}
