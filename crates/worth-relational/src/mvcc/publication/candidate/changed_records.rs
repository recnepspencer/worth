//! Descriptive prepared records stay under the actual candidate's live custody.
use super::PreparedRelationalCommitCandidate;

impl PreparedRelationalCommitCandidate {
    /// Borrows the exact, already-prepared touch graph without copying it.
    /// The callback must not reenter this candidate. Consumed or revoked
    /// candidates return `None`, distinct from an available exact-empty graph.
    /// The graph describes this candidate; it grants no source currentness.
    pub fn with_prepared_descriptive_touches<T>(
        &self,
        visit: impl FnOnce(&crate::history::data::RelationalDescriptiveTouchGraph) -> T,
    ) -> Option<T> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        payload
            .as_ref()
            .map(|payload| visit(payload.execution.descriptive_touches()))
    }

    /// Borrows the exact records already prepared for publication. The callback
    /// must not reenter this candidate. Consumed or revoked candidates have no
    /// prepared records; the callback is not invoked in that state.
    pub fn with_prepared_changed_records<T>(
        &self,
        visit: impl FnOnce(&[crate::transactions::data::RecordRef]) -> T,
    ) -> Option<T> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        payload
            .as_ref()
            .map(|payload| visit(payload.execution.changed_records()))
    }

    /// Exact record count already finalized by this prepared publication.
    /// A consumed or revoked candidate has no prepared payload to inspect.
    pub fn prepared_changed_record_count(&self) -> Option<usize> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|payload| payload.execution.changed_record_count())
    }
}
