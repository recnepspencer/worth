use crate::contracts::symbols::data::client_key::ClientKeyOwnerPort as _;
pub mod created_kinds;
use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractPlan;
use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::validation::engine::InvariantRuntimeView;

use super::budget::InvariantBudget;
use super::metrics::InvariantMetrics;
use super::observation::InvariantObservation;
use super::request::{PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes};
use super::state_view::InvariantStateView;

pub struct InvariantExecutionContext<'runtime, 'budget> {
    observation: InvariantObservation<'runtime>,
    version_id: crate::contracts::identity::data::primitives::VersionId,
    current_version_id: crate::contracts::identity::data::primitives::VersionId,
    semantic_plan: Option<RelationalSemanticPlan<'runtime>>,
    runtime: InvariantRuntimeView<'runtime>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    current_version_minimum_index:
        std::sync::Arc<std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>>,
    pub(super) created_lookup_profitable: bool,
    created_kinds: std::sync::OnceLock<Option<created_kinds::CreatedKinds<'runtime>>>,
    budget: Option<&'budget dyn InvariantBudget>,
}

impl<'runtime, 'budget> InvariantExecutionContext<'runtime, 'budget> {
    #[cfg(test)]
    pub fn new(
        runtime: &InvariantRuntimeView<'runtime>,
        observation: InvariantObservation<'runtime>,
        version_id: crate::contracts::identity::data::primitives::VersionId,
        current_version_id: crate::contracts::identity::data::primitives::VersionId,
        merged_plan: Option<&'runtime MergedCommitPlan>,
        relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
        current_version_minimum_index: std::sync::Arc<
            std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>,
        >,
    ) -> Self {
        Self::from_semantic_plan(
            runtime,
            observation,
            version_id,
            current_version_id,
            merged_plan.map(RelationalSemanticPlan::Mutation),
            relation_integrity_scopes,
            current_version_minimum_index,
        )
    }

    pub(crate) fn from_semantic_plan(
        runtime: &InvariantRuntimeView<'runtime>,
        observation: InvariantObservation<'runtime>,
        version_id: crate::contracts::identity::data::primitives::VersionId,
        current_version_id: crate::contracts::identity::data::primitives::VersionId,
        semantic_plan: Option<RelationalSemanticPlan<'runtime>>,
        relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
        current_version_minimum_index: std::sync::Arc<
            std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>,
        >,
    ) -> Self {
        Self {
            observation,
            version_id,
            current_version_id,
            semantic_plan,
            runtime: runtime.clone(),
            relation_integrity_scopes,
            current_version_minimum_index,
            created_lookup_profitable: false,
            created_kinds: std::sync::OnceLock::new(),
            budget: None,
        }
    }

    pub(crate) fn applied_created_entities(
        &self,
    ) -> Option<&'runtime crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings>{
        self.observation.applied_created_entities()
    }

    pub(crate) fn with_created_endpoint_scope(mut self, kind: KindId) -> Self {
        if let (Some(scopes), Some(plan)) = (&self.relation_integrity_scopes, self.merged_plan())
            && let Some(scope) = scopes.scope_for(kind)
        {
            let count = scopes.created_key_count() as u64;
            let levels = u64::from(u64::BITS - count.max(1).leading_zeros());
            let unique = scope.created_source_keys.max(scope.created_target_keys) as u64;
            // Distinct successful linear searches cost at least 1+...+U.
            // Index only when its whole construction and all lookup bounds
            // fit that lower bound, so it cannot add a work refusal.
            let old_minimum = unique.saturating_mul(unique + 1) / 2;
            let new_maximum = (plan.merged_intents.len() as u64)
                .saturating_add(count.saturating_mul(levels + 1))
                .saturating_add((scope.created_endpoint_count as u64).saturating_mul(levels + 1));
            let bytes = count.saturating_mul(std::mem::size_of::<(
                crate::contracts::identity::data::primitives::PartitionId,
                KindId,
                &crate::contracts::symbols::data::client_key::ClientKey,
            )>() as u64);
            let room = self.budget.and_then(InvariantBudget::remaining_scratch);
            self.created_lookup_profitable =
                count > 32 && new_maximum <= old_minimum && room.is_none_or(|room| bytes <= room);
        }
        self
    }

    pub(crate) fn with_budget(mut self, budget: &'budget dyn InvariantBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    pub(crate) fn is_leased(&self) -> bool {
        self.budget.is_some()
    }

    pub(crate) fn checkpoint(&self, units: u64) -> bool {
        self.budget.is_none_or(|budget| budget.checkpoint(units))
    }

    pub(crate) fn claim_result(&self, bytes: u64) -> bool {
        self.budget.is_none_or(|budget| budget.claim_result(bytes))
    }

    pub(crate) fn claim_contract_violation(
        &self,
        contract_id: &crate::contracts::schema::data::relation_integrity::contract_id::ContractId,
        references: &[&crate::contracts::transactions::data::primitives::EntityReference],
    ) -> bool {
        let reference_bytes = references
            .iter()
            .map(|reference| match reference {
                crate::contracts::transactions::data::primitives::EntityReference::Existing(_) => 0,
                crate::contracts::transactions::data::primitives::EntityReference::Created(
                    created,
                ) => created.client_key.owned_allocation_capacity_bytes(),
            })
            .sum::<u64>();
        self.claim_result(
            4096_u64
                .saturating_add((contract_id.as_str().len() as u64).saturating_mul(8))
                .saturating_add(reference_bytes.saturating_mul(8)),
        )
    }

    pub(crate) fn claim_scratch(&self, bytes: u64) -> bool {
        self.budget.is_none_or(|budget| budget.claim_scratch(bytes))
    }

    pub(crate) fn check_scratch_peak(&self, bytes: u64) -> bool {
        self.budget
            .is_none_or(|budget| budget.check_scratch_peak(bytes))
    }

    pub(crate) fn check_result_peak(&self, bytes: u64) -> bool {
        self.budget
            .is_none_or(|budget| budget.check_result_peak(bytes))
    }

    pub fn state_view(&self) -> InvariantStateView<'_> {
        InvariantStateView::new(
            self.observation.committed_partition_access(),
            self.version_id,
        )
    }

    pub fn partition_access(&self) -> &dyn worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess{
        self.observation.committed_partition_access()
    }

    pub(crate) fn enforcement_state_view(&self) -> InvariantStateView<'_> {
        InvariantStateView::new(
            self.observation.enforcement_partition_access(),
            self.observation.enforcement_version_id(self.version_id),
        )
    }

    pub fn current_version_id(&self) -> crate::contracts::identity::data::primitives::VersionId {
        self.current_version_id
    }

    pub fn merged_plan(&self) -> Option<&'runtime MergedCommitPlan> {
        self.semantic_plan.map(RelationalSemanticPlan::commands)
    }

    pub(crate) fn semantic_plan(&self) -> Option<RelationalSemanticPlan<'runtime>> {
        self.semantic_plan
    }

    pub(crate) fn current_version_minimum_index(
        &self,
    ) -> &std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex> {
        &self.current_version_minimum_index
    }

    pub fn relation_integrity_scope(
        &self,
        relation_kind_id: KindId,
    ) -> Option<&PreparedRelationIntegrityScope> {
        self.relation_integrity_scopes
            .as_ref()
            .and_then(|scopes| scopes.scope_for(relation_kind_id))
    }

    pub(crate) fn required_relation_integrity_scope(
        &self,
        relation_kind_id: KindId,
        class: crate::contracts::validation::data::execution::InvariantClass,
    ) -> Result<
        &PreparedRelationIntegrityScope,
        crate::contracts::validation::data::results::InvariantViolation,
    > {
        self.relation_integrity_scope(relation_kind_id)
            .ok_or_else(|| crate::contracts::validation::data::results::InvariantViolation {
                class,
                code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::PreparationFailure,
                detail: format!(
                    "required relation integrity scope for relation kind {:?} was not prepared",
                    relation_kind_id
                ),
                fields: crate::contracts::validation::data::results::InvariantViolationFields::None,
            })
    }

    pub(crate) fn entity_aspect_plan(&self, kind_id: KindId) -> Option<&LoweredAspectContractPlan> {
        self.runtime.entity_aspect_plan(kind_id)
    }

    pub(crate) fn unique_field_duplicate(
        &self,
        index: crate::contracts::indexes::data::definition::DerivedIndexId,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.runtime.unique_field_duplicate(index)
    }

    pub fn metrics(&self) -> InvariantMetrics<'runtime> {
        self.runtime.performance_access()
    }
}
