use super::WorthQueryAdmittedOperation;

impl WorthQueryAdmittedOperation {
    pub const fn operation_authority_identity_bytes(&self) -> &[u8; 32] {
        &self.data.operation_authority_identity_bytes
    }

    /// The admitted operation's definition identity. Unlike the authority
    /// identity it names no runtime, so idempotency intents bind it.
    pub const fn operation_definition_identity(&self) -> &[u8; 32] {
        &self.data.operation_definition_identity
    }

    pub const fn governed_input_identity(&self) -> Option<&[u8; 32]> {
        self.data.governed_input_identity.as_ref()
    }

    /// The single trusted admission instant read when this operation was admitted.
    pub const fn admitted_at(
        &self,
    ) -> Result<
        worth_foundational::facade::CanonicalTimestamp,
        crate::domain_computation::runtime_time::WorthQueryRuntimeTimeDenial,
    > {
        self.data.admitted_at
    }

    pub fn governed_proposal_identity(&self) -> Option<&[u8; 32]> {
        self.delegation_activation_binding()
            .map(|binding| binding.proposal_identity())
            .or_else(|| {
                self.capability_revocation_binding()
                    .map(|binding| binding.proposal_identity())
            })
    }
}
