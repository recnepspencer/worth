use crate::identity::data::KindId;
use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::engine::InvariantRuntimeView;

use super::metrics::InvariantMetrics;
use super::observation::InvariantObservation;
use super::request::{PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes};
use super::state_view::InvariantStateView;

pub struct InvariantExecutionContext<'runtime> {
    observation: InvariantObservation<'runtime>,
    version_id: crate::identity::data::VersionId,
    current_version_id: crate::identity::data::VersionId,
    merged_plan: Option<&'runtime MergedCommitPlan>,
    runtime: &'runtime InvariantRuntimeView<'runtime>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    current_version_minimum_index:
        std::sync::Arc<std::sync::OnceLock<super::evaluator::CurrentVersionMinimumIndex>>,
}

impl<'runtime> InvariantExecutionContext<'runtime> {
    pub fn new(
        runtime: &'runtime InvariantRuntimeView<'runtime>,
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
            runtime,
            relation_integrity_scopes,
            current_version_minimum_index,
        }
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

    pub fn relation_integrity_scope_budget(
        &self,
    ) -> &crate::config::data::RelationIntegrityScopeBudget {
        &self
            .runtime
            .config
            .execution
            .relation_integrity_scope_budget
    }
}
