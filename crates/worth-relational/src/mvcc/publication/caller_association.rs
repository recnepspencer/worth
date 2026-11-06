impl crate::mvcc::ValidatedRelationalProposal {
    /// Bind opaque caller meaning before the one canonical preparation.
    /// Consuming the proposal prevents a sealed candidate from being rebound.
    pub fn bind_caller_association(
        mut self,
        association: crate::mvcc::RelationalCallerAssociation,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        if self.state.caller_association.is_some() {
            return Err(super::origin_denial("caller association is already bound"));
        }
        use crate::durability::medium::RelationalHomeProgramWriteDenial as Denial;
        let home = worth_foundational::facade::HomeProgramTransitionCaller::recognizes(
            association.as_bytes(),
        );
        let denial = match (self.state.validation_input.home_step(), home) {
            (Some(step), true) if step.admits_association(association.as_bytes()) => None,
            (Some(_), true) | (Some(_), false) => Some(Denial::Admission(
                worth_proof::HomeProgramTransitionAdmissionDenial::CallerAssociationMismatch,
            )),
            (None, true) => Some(Denial::TransitionRequired),
            (None, false) => None,
        };
        if let Some(denial) = denial {
            return Err(
                crate::transactions::data::TransactionCommitError::publication_denied(
                    crate::mvcc::RelationalPublicationDenial::HomeProgramTransition(denial),
                ),
            );
        }
        self.state.caller_association = Some(association);
        Ok(self)
    }
}
