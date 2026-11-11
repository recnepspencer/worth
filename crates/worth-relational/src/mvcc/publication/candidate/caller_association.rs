impl super::PreparedRelationalCommitCandidate {
    /// The association in this candidate's sealed envelope. A revoked or
    /// consumed candidate has no retained publication payload to observe.
    ///
    /// ```compile_fail,E0599
    /// use worth_relational::facade::mvcc::{PreparedRelationalCommitCandidate, RelationalCallerAssociation};
    /// fn cannot_rebind(candidate: PreparedRelationalCommitCandidate, association: RelationalCallerAssociation) {
    ///     candidate.bind_caller_association(association);
    /// }
    /// ```
    pub fn caller_association(&self) -> Option<crate::mvcc::RelationalCallerAssociation> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| payload.execution.caller_association())
    }
}
