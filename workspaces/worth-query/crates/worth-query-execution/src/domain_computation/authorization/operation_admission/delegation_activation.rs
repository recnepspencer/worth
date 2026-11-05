use super::{ErasedAuthorizationBasis, WorthQueryAdmittedOperation};
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {crate::domain_computation::authorization::WorthQueryDelegationActivationBinding, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn retain_delegation_proposal_canonical_work(
        &mut self,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) {
        self.canonical_work = self.canonical_work.with_execution_work(work);
    }

    pub(super) fn bind_delegation_activation(
        mut self,
        binding: WorthQueryDelegationActivationBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        if !matches!(
            self.authorization_basis,
            ErasedAuthorizationBasis::Capability
        ) {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                &self.operation,
            ));
        }
        self.authorization_basis = ErasedAuthorizationBasis::DelegationActivation { binding };
        Ok(self)
    }

    pub(in crate::domain_computation) fn delegation_activation_binding(
        &self,
    ) -> Option<&WorthQueryDelegationActivationBinding> {
        match &self.authorization_basis {
            ErasedAuthorizationBasis::DelegationActivation { binding } => Some(binding),
            _ => None,
        }
    }
}
