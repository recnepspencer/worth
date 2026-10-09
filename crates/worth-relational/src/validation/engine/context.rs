use crate::identity::data::KindId;
use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::engine::InvariantRuntimeView;

use super::budget::InvariantBudget;
use super::metrics::InvariantMetrics;
use super::observation::InvariantObservation;
use super::request::{PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes};
use super::state_view::InvariantStateView;

pub struct InvariantExecutionContext<'runtime, 'budget> {
    observation: InvariantObservation<'runtime>,
    version_id: crate::identity::data::VersionId,
    current_version_id: crate::identity::data::VersionId,
    merged_plan: Option<&'runtime MergedCommitPlan>,
    runtime: InvariantRuntimeView<'runtime>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    current_version_minimum_index:
        std::sync::Arc<std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>>,
    budget: Option<&'budget dyn InvariantBudget>,
}

impl<'runtime, 'budget> InvariantExecutionContext<'runtime, 'budget> {
    pub fn new(
        runtime: &InvariantRuntimeView<'runtime>,
        observation: InvariantObservation<'runtime>,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'runtime MergedCommitPlan>,
        relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
        current_version_minimum_index: std::sync::Arc<
            std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>,
        >,
    ) -> Self {
        Self {
            observation,
            version_id,
            current_version_id,
            merged_plan,
            runtime: runtime.clone(),
            relation_integrity_scopes,
            current_version_minimum_index,
            budget: None,
        }
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
        contract_id: &crate::schema::data::ContractId,
        references: &[&crate::transactions::data::EntityReference],
    ) -> bool {
        let reference_bytes = references
            .iter()
            .map(|reference| match reference {
                crate::transactions::data::EntityReference::Existing(_) => 0,
                crate::transactions::data::EntityReference::Created(created) => {
                    created.client_key.owned_allocation_capacity_bytes()
                }
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

    pub fn partition_access(&self) -> &dyn crate::storage::overlay::PartitionAccess {
        self.observation.committed_partition_access()
    }

    pub(crate) fn enforcement_state_view(&self) -> InvariantStateView<'_> {
        InvariantStateView::new(
            self.observation.enforcement_partition_access(),
            self.observation.enforcement_version_id(self.version_id),
        )
    }

    pub fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.current_version_id
    }

    pub fn merged_plan(&self) -> Option<&'runtime MergedCommitPlan> {
        self.merged_plan
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
        class: crate::validation::data::InvariantClass,
    ) -> Result<&PreparedRelationIntegrityScope, crate::validation::data::InvariantViolation> {
        self.relation_integrity_scope(relation_kind_id)
            .ok_or_else(|| crate::validation::data::InvariantViolation {
                class,
                code: crate::diagnostics::data::DiagnosticCode::PreparationFailure,
                detail: format!(
                    "required relation integrity scope for relation kind {:?} was not prepared",
                    relation_kind_id
                ),
                fields: crate::validation::data::InvariantViolationFields::None,
            })
    }

    pub(crate) fn entity_aspect_plan(&self, kind_id: KindId) -> Option<&LoweredAspectContractPlan> {
        self.runtime.entity_aspect_plan(kind_id)
    }

    pub fn metrics(&self) -> InvariantMetrics<'runtime> {
        InvariantMetrics::new(self.runtime.performance_access())
    }
}
