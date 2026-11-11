use std::collections::BTreeSet;
mod created_bindings;
use created_bindings::{CreatedEntityKey, CreatedRelationKey};
use worth_execution::ExecutionOrderedMap;

use crate::authority::commit::preparation::planning::strategy::{
    ParallelLegality, ParallelProfitability, PreparationStrategy, PreparationStrategySelection,
};
use crate::config::data::MutationConfig;
use crate::identity::data::VersionId;
use crate::identity::data::{EntityId, RelationId};
use crate::runtime::MutationSymbolMeanings;
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
use crate::storage::overlay::WorkingState;

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

impl MutationPreparationTelemetry {
    pub(crate) fn accumulate(&mut self, part: Self) {
        self.packet_count += part.packet_count;
        self.packet_item_count += part.packet_item_count;
        self.packet_peak_width_total += part.packet_peak_width_total;
        self.scope_unit_count += part.scope_unit_count;
        self.parallel_legal_count += part.parallel_legal_count;
        self.parallel_profitable_count += part.parallel_profitable_count;
        self.serial_strategy_count += part.serial_strategy_count;
        self.staged_parallel_strategy_count += part.staged_parallel_strategy_count;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BranchLocalDeleteAllowance {
    pub(crate) entity_ids: BTreeSet<EntityId>,
    pub(crate) relation_ids: BTreeSet<RelationId>,
}

pub(crate) struct MutationWorkspace<'a> {
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    state: &'a mut WorkingState,
    symbols: &'a mut dyn MutationSymbolMeanings,
    config: &'a MutationConfig,
    schema: &'a RelationalSchemaRegistry,
    aspect_plans: &'a AspectContractPlanCatalog,
    version_id: VersionId,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    deferred_publication_edges: Option<&'a std::collections::BTreeSet<RelationId>>,
    preparation_telemetry: MutationPreparationTelemetry,
    created_entities: ExecutionOrderedMap<CreatedEntityKey<'a>, EntityId>,
    created_relations: ExecutionOrderedMap<CreatedRelationKey<'a>, RelationId>,
    record_allocations: Option<&'a mut dyn crate::runtime::MutationRecordAllocations>,
}

impl<'a> MutationWorkspace<'a> {
    pub(crate) fn new(
        execution: worth_execution::ExecutionRequest<'a, 'a>,
        state: &'a mut WorkingState,
        symbols: &'a mut dyn MutationSymbolMeanings,
        config: &'a MutationConfig,
        schema: &'a RelationalSchemaRegistry,
        aspect_plans: &'a AspectContractPlanCatalog,
        version_id: VersionId,
        branch_local_delete_allowance: BranchLocalDeleteAllowance,
        record_allocations: Option<&'a mut dyn crate::runtime::MutationRecordAllocations>,
        commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    ) -> Self {
        Self {
            execution,
            commit_work_budget,
            state,
            symbols,
            config,
            schema,
            aspect_plans,
            version_id,
            branch_local_delete_allowance,
            deferred_publication_edges: None,
            preparation_telemetry: MutationPreparationTelemetry::default(),
            created_entities: ExecutionOrderedMap::default(),
            created_relations: ExecutionOrderedMap::default(),
            record_allocations,
        }
    }

    pub(crate) fn execution(&self) -> worth_execution::ExecutionRequest<'a, 'a> {
        self.execution
    }
    pub(crate) fn defer_publication_edges(
        &mut self,
        edges: &'a std::collections::BTreeSet<RelationId>,
    ) {
        self.deferred_publication_edges = Some(edges);
    }

    pub(crate) fn commit_work_budget(&self) -> Option<&crate::execution::RequestWorkBudget> {
        self.commit_work_budget.as_ref()
    }

    /// Only owner mutation/assembly belongs here. Packet staging settles itself.
    pub(crate) fn run_owned_mutation<T, E>(
        &mut self,
        mutate: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::transactions::data::TransactionCommitError>,
    {
        let execution = self.execution;
        let budget = self.commit_work_budget.clone();
        crate::execution::transaction_stage(&execution, budget.as_ref(), || mutate(self))
    }

    pub(crate) fn with_context<R>(&mut self, f: impl FnOnce(MutationContext<'_>) -> R) -> R {
        let record_allocations = self
            .record_allocations
            .as_deref_mut()
            .expect(/* Proof: Only the authoritative creation workspace calls this method; that constructor supplies its record-allocation reservation, unlike read-only workspaces. */ "authoritative record creation requires allocation authority");
        f(MutationContext {
            execution: self.execution,
            state: self.state,
            schema: self.schema,
            record_allocations,
            deferred_publication_edges: self.deferred_publication_edges,
        })
    }

    pub(crate) fn write_delta_revision_stamps(
        &mut self,
        delta: &super::canonical_deltas::CanonicalRecordAspectDelta,
        version: VersionId,
    ) -> Result<(), super::MutationApplyDenial> {
        let admitted = super::delta_symbols::AdmittedDeltaSymbols::admit(delta, self.symbols)?;
        super::aspect_versions::write_aspect_versions_for_delta_for_request(
            &self.execution,
            self.state,
            &admitted,
            version,
        )
        .map_err(|cause| {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        })?;
        super::field_versions::write_field_versions_for_delta(
            &self.execution,
            self.state,
            &admitted,
            version,
        )
        .map_err(|cause| {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        })?;
        Ok(())
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
}

impl crate::capabilities::AspectPlanSource for MutationWorkspace<'_> {
    fn aspect_plan_catalog(&self) -> &AspectContractPlanCatalog {
        self.aspect_plans
    }
    fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        MutationWorkspace::entity_aspect_plan(self, kind_id)
    }
    fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        MutationWorkspace::relation_aspect_plan(self, kind_id)
    }
}
