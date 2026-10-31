impl crate::mvcc::ValidatedRelationalProposal {
    /// Bind opaque caller meaning before the one canonical preparation.
    /// Consuming the proposal prevents a sealed candidate from being rebound.
    pub fn bind_caller_association(
        mut self,
        association: crate::mvcc::RelationalCallerAssociation,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        if self.caller_association.is_some() {
            return Err(super::origin_denial("caller association is already bound"));
        }
        self.caller_association = Some(association);
        Ok(self)
    }
}
