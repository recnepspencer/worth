use std::collections::{BTreeMap, BTreeSet};

use crate::authority::commit::preparation::planning::strategy::{
    ParallelLegality, ParallelProfitability, PreparationStrategy, PreparationStrategySelection,
};
use crate::config::data::MutationConfig;
use crate::identity::data::VersionId;
use crate::identity::data::{EntityId, RelationId};
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
use crate::storage::overlay::WorkingState;
use crate::symbols::data::StringInterner;
use crate::transactions::data::{CreatedEntityRef, CreatedRelationRef, EntityReference};

use super::mutation_context::MutationContext;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MutationPreparationTelemetry {
    pub(crate) packet_count: usize,
    pub(crate) packet_item_count: usize,
    pub(crate) packet_peak_width_total: usize,
    pub(crate) scope_unit_count: usize,
    pub(crate) parallel_legal_count: usize,
    pub(crate) parallel_profitable_count: usize,
    pub(crate) serial_strategy_count: usize,
    pub(crate) staged_parallel_strategy_count: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BranchLocalDeleteAllowance {
    pub(crate) entity_ids: BTreeSet<EntityId>,
    pub(crate) relation_ids: BTreeSet<RelationId>,
}

pub(crate) struct MutationWorkspace<'a> {
    commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    state: &'a mut WorkingState,
    symbols: &'a mut StringInterner,
    config: &'a MutationConfig,
    schema: &'a RelationalSchemaRegistry,
    aspect_plans: &'a AspectContractPlanCatalog,
    version_id: VersionId,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    preparation_telemetry: MutationPreparationTelemetry,
    created_entities: BTreeMap<CreatedEntityRef, EntityId>,
    created_relations: BTreeMap<CreatedRelationRef, RelationId>,
    record_allocations: Option<&'a mut crate::runtime::PendingRecordAllocations>,
}

impl<'a> MutationWorkspace<'a> {
    pub(crate) fn new(
        state: &'a mut WorkingState,
        symbols: &'a mut StringInterner,
        config: &'a MutationConfig,
        schema: &'a RelationalSchemaRegistry,
        aspect_plans: &'a AspectContractPlanCatalog,
        version_id: VersionId,
        branch_local_delete_allowance: BranchLocalDeleteAllowance,
        record_allocations: Option<&'a mut crate::runtime::PendingRecordAllocations>,
        commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    ) -> Self {
        Self {
            commit_work_budget,
            state,
            symbols,
            config,
            schema,
            aspect_plans,
            version_id,
            branch_local_delete_allowance,
            preparation_telemetry: MutationPreparationTelemetry::default(),
            created_entities: BTreeMap::new(),
            created_relations: BTreeMap::new(),
            record_allocations,
        }
    }

    pub(crate) fn commit_work_budget(&self) -> Option<&crate::execution::RequestWorkBudget> {
        self.commit_work_budget.as_ref()
    }

    pub(crate) fn with_context<R>(&mut self, f: impl FnOnce(MutationContext<'_>) -> R) -> R {
        let record_allocations = self
            .record_allocations
            .as_deref_mut()
            .expect("authoritative record creation requires allocation authority");
        f(MutationContext {
            state: self.state,
            symbols: self.symbols,
            schema: self.schema,
            record_allocations,
        })
    }

    pub(crate) fn cascade_delete_policy(&self) -> crate::config::data::CascadeDeletePolicy {
        self.config.cascade_delete_policy
    }

    pub(crate) fn version_id(&self) -> VersionId {
        self.version_id
    }

    pub(crate) fn branch_local_delete_allows_entity(&self, entity_id: EntityId) -> bool {
        self.branch_local_delete_allowance
            .entity_ids
            .contains(&entity_id)
    }

    pub(crate) fn branch_local_delete_allows_relation(&self, relation_id: RelationId) -> bool {
        self.branch_local_delete_allowance
            .relation_ids
            .contains(&relation_id)
    }

    pub(crate) fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.entity_plans.get(&kind_id)
    }

    pub(crate) fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.relation_plans.get(&kind_id)
    }

    pub(crate) fn record_preparation_strategy(
        &mut self,
        packet_count: usize,
        packet_item_count: usize,
        packet_max_width: usize,
        scope_unit_count: usize,
        strategy: PreparationStrategy,
    ) {
        self.preparation_telemetry.packet_count += packet_count;
        self.preparation_telemetry.packet_item_count += packet_item_count;
        self.preparation_telemetry.packet_peak_width_total += packet_max_width;
        self.preparation_telemetry.scope_unit_count += scope_unit_count;
        if matches!(strategy.parallel_legality, ParallelLegality::ProvenParallel) {
            self.preparation_telemetry.parallel_legal_count += 1;
        }
        if matches!(
            strategy.parallel_profitability,
            ParallelProfitability::Profitable
        ) {
            self.preparation_telemetry.parallel_profitable_count += 1;
        }
        match strategy.selected_mode {
            PreparationStrategySelection::Serial => {
                self.preparation_telemetry.serial_strategy_count += 1;
            }
            PreparationStrategySelection::StagedParallel => {
                self.preparation_telemetry.staged_parallel_strategy_count += 1;
            }
        }
    }

    pub(crate) fn preparation_telemetry(&self) -> MutationPreparationTelemetry {
        self.preparation_telemetry
    }

    pub(crate) fn into_created_bindings(
        self,
    ) -> (
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
    ) {
        let MutationWorkspace {
            created_entities,
            created_relations,
            symbols,
            ..
        } = self;
        let created_entities =
            crate::transactions::data::CommitCreatedEntityBindings::from_owner_map(
                created_entities,
                symbols,
            );
        let created_relations =
            crate::transactions::data::CommitCreatedRelationBindings::from_owner_map(
                created_relations,
                symbols,
            );
        (created_entities, created_relations)
    }

    pub(crate) fn register_created_entity(
        &mut self,
        created: CreatedEntityRef,
        entity_id: EntityId,
    ) {
        self.created_entities.insert(created, entity_id);
    }

    pub(crate) fn register_created_relation(
        &mut self,
        created: CreatedRelationRef,
        relation_id: RelationId,
    ) {
        self.created_relations.insert(created, relation_id);
    }

    pub(crate) fn resolve_entity_reference(
        &self,
        entity_reference: &EntityReference,
    ) -> Option<EntityId> {
        match entity_reference {
            EntityReference::Existing(entity_id) => Some(*entity_id),
            EntityReference::Created(created) => self.created_entities.get(created).copied(),
        }
    }
}
