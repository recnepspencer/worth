use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use std::collections::BTreeSet;
pub mod created_bindings;
use created_bindings::{CreatedEntityKey, CreatedRelationKey};
use worth_execution::ExecutionOrderedMap;

use crate::contracts::config::data::policies::MutationConfig;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::mutation_input::MutationSymbolMeanings;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::{AspectContractPlanCatalog, LoweredAspectContractPlan};
use crate::contracts::schema::data::RelationalSchemaRegistry;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use super::mutation_context::MutationContext;

#[derive(Debug, Clone, Default)]
pub struct BranchLocalDeleteAllowance {
    pub entity_ids: BTreeSet<EntityId>,
    pub relation_ids: BTreeSet<RelationId>,
}

pub(crate) struct MutationWorkspace<'a> {
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    commit_work_budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
    state: &'a mut WorkingState,
    symbols: &'a mut dyn MutationSymbolMeanings,
    config: &'a MutationConfig,
    schema: &'a RelationalSchemaRegistry,
    aspect_plans: &'a AspectContractPlanCatalog,
    version_id: VersionId,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    deferred_publication_edges: Option<&'a std::collections::BTreeSet<RelationId>>,
    created_entities: ExecutionOrderedMap<CreatedEntityKey<'a>, EntityId>,
    created_relations: ExecutionOrderedMap<CreatedRelationKey<'a>, RelationId>,
    record_allocations: Option<
        &'a mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    >,
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
        record_allocations: Option<&'a mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations>,
        commit_work_budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
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

    /// Charge mutation and effect assembly to the original request.
    pub(crate) fn run_owned_mutation<T, E>(
        &mut self,
        mutate: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<
            crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
        >,
{
        let execution = self.execution;
        let budget = self.commit_work_budget.clone();
        crate::contracts::execution::request_work::transaction_stage(
            &execution,
            budget.as_ref(),
            || mutate(self),
        )
    }

    /// Name the records `intent` allocates, so the allocation evidence can
    /// follow the envelope's merged plan (`PendingRecordAllocations::reorder_live`).
    pub(crate) fn begin_intent(
        &mut self,
        intent: &crate::contracts::transactions::data::intents::mutation_intent::MutationIntent,
    ) {
        let sources =
            worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_source::AllocationSource::of(intent);
        if let (false, Some(allocations)) =
            (sources.is_empty(), self.record_allocations.as_deref_mut())
        {
            allocations.begin_intent(sources);
        }
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
            crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
        })?;
        super::field_versions::write_field_versions_for_delta(
            &self.execution,
            self.state,
            &admitted,
            version,
        )
        .map_err(|cause| {
            crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
        })?;
        Ok(())
    }

    pub(crate) fn cascade_delete_policy(
        &self,
    ) -> crate::contracts::config::data::policies::CascadeDeletePolicy {
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

    pub(crate) fn prepare_entity_field_update(
        &self,
        intent: &crate::contracts::transactions::data::intents::mutation_intent::UpdateEntityFieldsIntent,
    ) -> Result<
        super::intents::entity_aspect_preparation::PreparedEntityAspectMutation,
        crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >{
        super::intents::entity_aspect_preparation::prepare_fields(
            &self.execution(),
            intent,
            self.state,
            self.aspect_plans,
        )
    }

    pub(crate) fn prepare_entity_aspect_patch(
        &self,
        intent: &crate::contracts::transactions::data::intents::mutation_intent::ApplyEntityAspectPatchIntent,
    ) -> Result<
        super::intents::entity_aspect_preparation::PreparedEntityAspectMutation,
        crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >{
        super::intents::entity_aspect_preparation::prepare_portable(
            &self.execution(),
            intent,
            self.state,
            self.aspect_plans,
        )
    }

    pub(crate) fn entity_aspect_plan(
        &self,
        kind_id: crate::contracts::identity::data::primitives::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.entity_plans.get(&kind_id)
    }

    pub(crate) fn relation_aspect_plan(
        &self,
        kind_id: crate::contracts::identity::data::primitives::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.relation_plans.get(&kind_id)
    }
}

impl crate::contracts::schema::aspect_plan_source::AspectPlanSource for MutationWorkspace<'_> {
    fn aspect_plan_catalog(&self) -> &AspectContractPlanCatalog {
        self.aspect_plans
    }
    fn entity_aspect_plan(
        &self,
        kind_id: crate::contracts::identity::data::primitives::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        MutationWorkspace::entity_aspect_plan(self, kind_id)
    }
    fn relation_aspect_plan(
        &self,
        kind_id: crate::contracts::identity::data::primitives::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        MutationWorkspace::relation_aspect_plan(self, kind_id)
    }
}
