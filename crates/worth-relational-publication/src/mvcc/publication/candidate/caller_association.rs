impl super::PreparedRelationalCommitCandidate {
    /// The association in this candidate's sealed envelope. A revoked or
    /// consumed candidate has no retained publication payload to observe.
    pub fn caller_association(&self) -> Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>{
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| payload.execution.caller_association())
    }
}
