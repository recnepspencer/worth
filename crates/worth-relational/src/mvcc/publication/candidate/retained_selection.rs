//! Publication profile follows the actual prepared source, not a descriptor.
use super::PreparedRelationalCommitCandidate;

impl PreparedRelationalCommitCandidate {
    pub(in crate::mvcc::publication) fn requires_retained_selection_profile(&self) -> bool {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|payload| payload.execution.is_retained_entity_selection())
    }
}
