//! Publication profile follows the actual prepared source, not a descriptor.
use super::PreparedRelationalCommitCandidate;
use worth_relational_contracts::internal_port as contracts;

impl PreparedRelationalCommitCandidate {
    pub(in crate::internal_port::mvcc::publication) fn owned_graph_selection_model(
        &self,
    ) -> Option<contracts::identity::data::record_ids::EntityId> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| payload.execution.owned_graph_selection_model())
    }
    pub(in crate::internal_port::mvcc::publication) fn requires_retained_selection_profile(
        &self,
    ) -> bool {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|payload| payload.execution.is_retained_entity_selection())
    }
}

#[doc(hidden)]
pub trait PreparedRelationalCommitCandidateOwnerPort: Sized {
    fn owned_graph_selection_model(
        &self,
    ) -> Option<contracts::identity::data::record_ids::EntityId>;

    fn requires_retained_selection_profile(&self) -> bool;
}

impl PreparedRelationalCommitCandidateOwnerPort for PreparedRelationalCommitCandidate {
    fn owned_graph_selection_model(
        &self,
    ) -> Option<contracts::identity::data::record_ids::EntityId> {
        self.owned_graph_selection_model()
    }

    fn requires_retained_selection_profile(&self) -> bool {
        self.requires_retained_selection_profile()
    }
}
