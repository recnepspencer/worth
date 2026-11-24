use crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary;
use crate::transactions::data::primitives::TransactionId;
use crate::validation::data::contracts::InvariantPlanContract;
use crate::validation::data::execution::InvariantExecutionPoint;
use crate::validation::engine::InvariantObservationKind;
use worth_relational_contracts::internal_port::validation::data::execution::InvariantExecutionPointOwnerPort as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparationPlanningContext {
    pub(crate) transaction_id: Option<TransactionId>,
    pub(crate) execution_point: InvariantExecutionPoint,
    pub(crate) observation_kind: InvariantObservationKind,
    pub(crate) version_id: crate::identity::data::VersionId,
    pub(crate) current_version_id: crate::identity::data::VersionId,
    pub(crate) structural_summary: Option<CommitStructuralSummary>,
    pub(crate) plan_contract: Option<InvariantPlanContract>,
    pub(crate) schema_registry_entry_count: usize,
    pub(crate) invariant_registration_count: usize,
}

impl PreparationPlanningContext {
    /// Specialize the shared proposal proof to one declared family's read view.
    /// Every other authority/basis field is preserved exactly.
    pub(crate) fn for_registration(&self, point: InvariantExecutionPoint) -> Option<Self> {
        if !self.execution_point.includes(point) {
            return None;
        }
        let mut selected = self.clone();
        if self.execution_point == InvariantExecutionPoint::ProposedState {
            selected.execution_point = point;
            if point == InvariantExecutionPoint::CommitBoundary {
                selected.observation_kind = InvariantObservationKind::Committed;
                selected.version_id = self.current_version_id;
            } else {
                selected.observation_kind = InvariantObservationKind::Speculative;
            }
        }
        Some(selected)
    }
}
